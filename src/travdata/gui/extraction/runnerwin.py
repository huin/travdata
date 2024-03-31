# -*- coding: utf-8 -*-
"""Defines a window that monitors and manages extraction from a PDF."""

# Pylint doesn't like QT much.
# pylint: disable=I1101

from typing import Optional

from PySide6 import QtCore, QtWidgets

from travdata import config
from travdata.extraction import pdfextract
from travdata.gui import qtutil


class _WorkerSignals(QtCore.QObject):
    progress = QtCore.Signal(pdfextract.Progress)
    error = QtCore.Signal(str)
    stopped = QtCore.Signal()
    finished = QtCore.Signal()


class _Worker(QtCore.QRunnable):

    def __init__(
        self,
        cfg: pdfextract.ExtractionConfig,
        table_reader: pdfextract.TableReader,
    ) -> None:
        super().__init__()
        self.signals = _WorkerSignals()
        self._cfg = cfg
        self._table_reader = table_reader
        self._continue = True

    def stop(self) -> None:
        """Stops extraction as soon as possible."""
        self._continue = False

    @QtCore.Slot()
    def run(self) -> None:
        """Runs the extraction."""
        try:
            if self._cfg.book_cfg.group is None:
                cfg = config.load_config(self._cfg.config_dir, [self._cfg.book_cfg.id_])
                self._cfg.book_cfg = cfg.books[self._cfg.book_cfg.id_]

            pdfextract.extract_book(
                table_reader=self._table_reader,
                cfg=self._cfg,
                events=pdfextract.ExtractEvents(
                    on_error=self.signals.error.emit,
                    on_progress=self.signals.progress.emit,
                    do_continue=lambda: self._continue,
                ),
            )
        except Exception as exc:  # pylint: disable=broad-exception-caught
            self.signals.error.emit(str(exc))
            self.signals.stopped.emit()
        else:
            self.signals.finished.emit()


class ExtractionRunnerWindow(QtWidgets.QWidget):
    """Window to manage extraction from PDF."""

    closing = QtCore.Signal()

    _worker: Optional[_Worker]

    def __init__(
        self,
        cfg: pdfextract.ExtractionConfig,
        thread_pool: QtCore.QThreadPool,
        table_reader: pdfextract.TableReader,
        *args,
        **kwargs,
    ) -> None:
        super().__init__(*args, **kwargs)
        self.setWindowTitle("Travdata Extraction")

        self._worker = None

        self._cfg = cfg
        self._thread_pool = thread_pool
        self._table_reader = table_reader

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
        self._worker = _Worker(self._cfg, self._table_reader)
        self._worker.signals.progress.connect(self._progress)
        self._worker.signals.error.connect(self._error)
        self._worker.signals.finished.connect(self._finished)
        self._worker.signals.stopped.connect(self._stopped)
        self._thread_pool.start(self._worker)

    def stop_extraction(self) -> None:
        """Stops the extraction as soon as possible."""
        if self._worker is None:
            return
        self._worker.stop()
        self._worker = None
        self._cancel_button.setEnabled(False)

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
    def _progress(self, progress: pdfextract.Progress) -> None:
        if progress.total != self._progress_bar.maximum():
            self._progress_bar.setMaximum(progress.total)
        self._progress_bar.setValue(progress.completed)

    @QtCore.Slot()
    def _error(self, error: str) -> None:
        self._output_text_area.appendPlainText(error + "\n")

    @QtCore.Slot()
    def _finished(self) -> None:
        self._output_text_area.appendPlainText("Complete.")
        self._cancel_button.setEnabled(False)

    @QtCore.Slot()
    def _stopped(self) -> None:
        self._output_text_area.appendPlainText("Stopped.")
        self._cancel_button.setEnabled(False)
