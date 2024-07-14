# -*- coding: utf-8 -*-
"""Defines a window that monitors and manages extraction from a PDF."""

# Pylint doesn't like QT much.
# pylint: disable=I1101

import multiprocessing
import multiprocessing.connection
import traceback
from typing import Iterator, Optional

from PySide6 import QtCore, QtWidgets

from travdata.extraction import bookextract
from travdata.extraction.pdf import cachingreader, tabulareader
from travdata.gui import qtutil


def _unhandled_exception_events(
    exc: Exception,
) -> list[bookextract.ExtractEvent]:
    details = "".join(traceback.format_exception(exc))
    return [
        bookextract.ErrorEvent(
            message=f"Unhandled exception during extraction: {details}",
        ),
        bookextract.EndedEvent(abnormal=True),
    ]


def _worker(
    conn: multiprocessing.connection.Connection,
    ext_cfg: bookextract.ExtractionConfig,
):
    """Performs the configured extraction.

    Runs as the target of a ``multiprocessing.Process``.
    """
    with conn:
        try:
            with (
                tabulareader.TabulaClient(
                    force_subprocess=False,
                ) as tabula_client,
                cachingreader.optional_table_cache(
                    delegate=tabula_client,
                    disable=False,
                ) as table_reader,
            ):
                do_continue: bool = True

                events = bookextract.extract_book(
                    table_reader=table_reader,
                    ext_cfg=ext_cfg,
                    do_continue=lambda: do_continue,
                )

                for event in events:
                    if conn.poll():
                        conn.recv()
                        # Currently the only message is implicitly "cancel".
                        do_continue = False
                    conn.send(event)

        except Exception as exc:  # pylint: disable=broad-exception-caught
            # This most likely catches exceptions from the context managers
            # outside the call to extract_book.
            for event in _unhandled_exception_events(exc):
                conn.send(event)


class _WorkerSignals(QtCore.QObject):
    events = QtCore.Signal(bookextract.ExtractEvent)


class _Supervisor(QtCore.QRunnable):
    signals: _WorkerSignals

    def __init__(
        self,
        ext_cfg: bookextract.ExtractionConfig,
    ) -> None:
        super().__init__()
        self.signals = _WorkerSignals()
        self._ext_cfg = ext_cfg
        self._continue = True
        self._worker_conn, self._supervisor_conn = multiprocessing.Pipe(duplex=True)

    def stop(self) -> None:
        """Stops extraction as soon as possible."""
        self._supervisor_conn.send(None)

    def _iter_events(self, proc: multiprocessing.Process) -> Iterator[bookextract.ExtractEvent]:
        got_end: bool = False
        while True:
            # Healthcheck the worker frequently (every 0.1 seconds after
            # not receiving an event).
            while not self._supervisor_conn.poll(0.1):
                if not proc.is_alive():
                    if got_end:
                        return
                    raise RuntimeError(
                        f"Extraction process unexpectedly exited with code {proc.exitcode}."
                    )

            try:
                event = self._supervisor_conn.recv()
            except EOFError:
                return

            match event:
                case bookextract.EndedEvent():
                    got_end = True

            yield event

    @QtCore.Slot()
    def run(self) -> None:
        """Runs the extraction."""
        try:
            proc = multiprocessing.Process(
                target=_worker,
                kwargs={
                    "conn": self._worker_conn,
                    "ext_cfg": self._ext_cfg,
                },
            )

            proc.start()
            try:
                for event in self._iter_events(proc):
                    self.signals.events.emit(event)
            finally:
                proc.join()

        except Exception as exc:  # pylint: disable=broad-exception-caught
            for event in _unhandled_exception_events(exc):
                self.signals.events.emit(event)


class ExtractionRunnerWindow(QtWidgets.QWidget):
    """Window to manage extraction from PDF."""

    closing = QtCore.Signal()

    _supervisor: Optional[_Supervisor]

    def __init__(
        self,
        ext_cfg: bookextract.ExtractionConfig,
        thread_pool: QtCore.QThreadPool,
        *args,
        **kwargs,
    ) -> None:
        super().__init__(*args, **kwargs)
        self.setWindowTitle("Travdata Extraction")

        self._supervisor = None

        self._ext_cfg = ext_cfg
        self._thread_pool = thread_pool

        self._output_text_area = QtWidgets.QPlainTextEdit()
        self._output_text_area.setReadOnly(True)

        self._progress_bar = QtWidgets.QProgressBar()
        self._progress_bar.setMinimum(0)

        self._cancel_button = QtWidgets.QPushButton("Cancel")
        self._cancel_button.clicked.connect(self._cancel)

        contents = qtutil.make_group_vbox(
            "Extraction progress",
            self._output_text_area,
            self._progress_bar,
            self._cancel_button,
        )

        layout = QtWidgets.QStackedLayout()
        layout.addWidget(contents)
        self.setLayout(layout)

    def start_extraction(self) -> None:
        """Starts the extraction."""
        try:
            self._supervisor = _Supervisor(self._ext_cfg)
            self._supervisor.signals.events.connect(self._extraction_event)
            self._thread_pool.start(self._supervisor)
        except Exception as exc:  # pylint: disable=broad-exception-caught
            self.stop_extraction()
            self._output_text_area.appendPlainText("".join(traceback.format_exception(exc)))

    def stop_extraction(self) -> None:
        """Stops the extraction as soon as possible."""
        self._cancel_button.setEnabled(False)
        if self._supervisor is None:
            return
        self._supervisor.stop()
        self._supervisor = None

    def closeEvent(self, event) -> None:  # pylint: disable=invalid-name
        """Captures event of window closing."""
        self.stop_extraction()
        self.closing.emit()
        super().closeEvent(event)

    @QtCore.Slot()
    def _cancel(self) -> None:
        self._output_text_area.appendPlainText("Cancelling...")
        self.stop_extraction()

    @QtCore.Slot()
    def _extraction_event(self, event: bookextract.ExtractEvent) -> None:
        match event:
            case bookextract.EndedEvent(abnormal=True):
                self._output_text_area.appendPlainText("Failed.")
                self._cancel_button.setEnabled(False)
            case bookextract.EndedEvent(abnormal=False):
                self._output_text_area.appendPlainText("Complete.")
                self._cancel_button.setEnabled(False)
            case bookextract.ErrorEvent(message):
                self._output_text_area.appendPlainText(message)
            case bookextract.FileOutputEvent(path):
                self._output_text_area.appendPlainText(f"Output {path}")
            case bookextract.ProgressEvent(completed, total):
                if total != self._progress_bar.maximum():
                    self._progress_bar.setMaximum(total)
                self._progress_bar.setValue(completed)
