# -*- coding: utf-8 -*-
"""Defines a window that configures extraction from a PDF."""

# Pylint doesn't like QT much.
# pylint: disable=I1101

import dataclasses
import pathlib
from typing import Callable, Optional

from PySide6 import QtCore, QtWidgets, QtGui

from travdata import commontext, config
from travdata.extraction import pdfextract
from travdata.gui import qtutil
from travdata.gui.extraction import runnerwin


@dataclasses.dataclass
class _ExtractionConfigErrors:
    config_dir: Optional[str] = None
    input_pdf: Optional[str] = None
    book_id: Optional[str] = None
    output_dir: Optional[str] = None


@dataclasses.dataclass
class _ExtractionConfigBuilder:
    cfg: Optional[config.Config] = None
    cfg_error: Optional[str] = None

    # Remaining fields are Optional variations of those in ExtractionConfig.
    config_dir: Optional[pathlib.Path] = None
    input_pdf: Optional[pathlib.Path] = None
    book_id: Optional[str] = None
    output_dir: Optional[pathlib.Path] = None

    def __post_init__(self) -> None:
        self.set_config_dir(self.config_dir, force_update=True)

    def set_config_dir(
        self,
        path: Optional[pathlib.Path],
        *,
        force_update: bool = False,
    ) -> bool:
        """Sets the config dir, loading the config if set.

        :param path: New path to the configuration directory.
        :param force_update: Forces an update, even if it appears that there
        would be no change.
        :return: True if changed.
        """
        if not force_update and self.config_dir == path:
            return False

        self.config_dir = path
        if self.config_dir is None:
            self.cfg = None
        else:
            try:
                cfg = config.load_config(self.config_dir, limit_books=[])
            except OSError as exc:
                self.cfg = None
                self.cfg_error = str(exc)
            else:
                self.cfg = cfg
                self.cfg_error = None

        if self.cfg is None or self.book_id not in self.cfg.books:
            self.book_id = None

        return True

    def build_errors(self) -> _ExtractionConfigErrors:
        """Returns any errors in the builder (other than unspecified values)."""
        errors = _ExtractionConfigErrors()
        if self.cfg is None:
            errors.config_dir = "Configuration must be selected."

        if self.cfg_error is not None:
            errors.config_dir = self.cfg_error

        if self.input_pdf is not None:
            if not self.input_pdf.exists():
                errors.input_pdf = f"{self.input_pdf} does not exist."
            elif not self.input_pdf.is_file():
                errors.input_pdf = f"{self.input_pdf} is not a regular file."

        if self.output_dir is not None:
            if not self.output_dir.exists():
                errors.input_pdf = f"{self.output_dir} does not exist."
            elif not self.output_dir.is_dir():
                errors.input_pdf = f"{self.output_dir} is not a directory."

        return errors

    def build(self) -> Optional[pdfextract.ExtractionConfig]:
        """Builds the extraction configuration, if complete."""
        if self.cfg is None:
            return None
        if self.config_dir is None:
            return None
        if self.input_pdf is None:
            return None
        if self.book_id is None:
            return None
        if self.output_dir is None:
            return None

        return pdfextract.ExtractionConfig(
            config_dir=self.config_dir,
            output_dir=self.output_dir,
            input_pdf=self.input_pdf,
            book_cfg=self.cfg.books[self.book_id],
            overwrite_existing=False,
        )


class ExtractionConfigWindow(QtWidgets.QMainWindow):  # pylint: disable=too-many-instance-attributes
    """QT window to configure and start PDF extraction."""

    # _extract_builder and _extract contain the data model, separately from any
    # widgets.
    _extract_builder: _ExtractionConfigBuilder
    _extract: Optional[pdfextract.ExtractionConfig]

    _runner: Optional[runnerwin.ExtractionRunnerWindow]

    def __init__(
        self,
        thread_pool: QtCore.QThreadPool,
        table_reader: pdfextract.TableReader,
        config_dir: Optional[pathlib.Path],
    ) -> None:
        super().__init__()
        self.setWindowTitle("Travdata Extraction Setup")

        data_usage_text = QtWidgets.QLabel(commontext.DATA_USAGE)

        self._thread_pool = thread_pool
        self._table_reader = table_reader

        self._runner = None

        self._extract_builder = _ExtractionConfigBuilder(
            config_dir=config_dir,
        )
        self._extract = None

        self._config_dir_label = QtWidgets.QLabel("")
        self._config_dir_error = QtWidgets.QLabel("")
        self._config_dir_button = QtWidgets.QPushButton("Select configuration")
        self._config_dir_button.clicked.connect(self._select_config_dir)

        self._input_pdf_label = QtWidgets.QLabel("")
        self._input_pdf_error = QtWidgets.QLabel("")
        self._input_pdf_button = QtWidgets.QPushButton("Select PDF")
        self._input_pdf_button.clicked.connect(self._select_input_pdf)

        self._book_combo_dirty = True
        self._book_combo = QtWidgets.QComboBox()
        self._book_combo.currentIndexChanged.connect(self._select_book)
        self._book_error = QtWidgets.QLabel("")

        self._output_dir_label = QtWidgets.QLabel("")
        self._output_dir_error = QtWidgets.QLabel("")
        self._output_dir_button = QtWidgets.QPushButton("Select output directory")
        self._output_dir_button.clicked.connect(self._select_output_dir)

        config_box = qtutil.make_group_vbox(
            "Extraction configuration",
            self._config_dir_label,
            self._config_dir_error,
            self._config_dir_button,
        )

        input_pdf_box = qtutil.make_group_vbox(
            "Input PDF",
            self._input_pdf_label,
            self._input_pdf_error,
            self._input_pdf_button,
            self._book_combo,
            self._book_error,
        )

        output_dir_box = qtutil.make_group_vbox(
            "Output directory",
            self._output_dir_label,
            self._output_dir_error,
            self._output_dir_button,
        )

        self._extract_button = QtWidgets.QPushButton("Extract")
        self._extract_button.clicked.connect(self._run_extraction)

        outer_box = qtutil.make_group_vbox(
            "Extract tables from PDF",
            data_usage_text,
            config_box,
            input_pdf_box,
            output_dir_box,
            QtWidgets.QSpacerItem(
                0,
                0,
                QtWidgets.QSizePolicy.Policy.MinimumExpanding,
                QtWidgets.QSizePolicy.Policy.MinimumExpanding,
            ),
            self._extract_button,
        )

        self.setCentralWidget(outer_box)

    def showEvent(self, event: QtGui.QShowEvent) -> None:  # pylint: disable=invalid-name
        """Intercepts the window being shown."""
        self._refresh_from_state()
        return super().showEvent(event)

    def _refresh_from_state(self) -> None:
        """Update widgets from current self.state."""
        _bulk_enable(
            self._extract_builder.cfg is not None,
            self._input_pdf_button,
            self._book_combo,
            self._output_dir_button,
        )
        _update_path_label(self._config_dir_label, self._extract_builder.config_dir)
        _update_path_label(self._input_pdf_label, self._extract_builder.input_pdf)
        if self._book_combo_dirty:
            _repopulate_book_combo(self._book_combo, self._extract_builder.cfg)
            self._book_combo_dirty = False
        _update_book_combo(self._book_combo, self._extract_builder.book_id)
        _update_path_label(self._output_dir_label, self._extract_builder.output_dir)

        errors = self._extract_builder.build_errors()
        _update_error_label(self._config_dir_error, errors.config_dir)
        _update_error_label(self._input_pdf_error, errors.input_pdf)
        _update_error_label(self._book_error, errors.book_id)
        _update_error_label(self._output_dir_error, errors.output_dir)

        self._extract = self._extract_builder.build()
        self._extract_button.setEnabled(self._extract is not None and self._runner is None)

    @QtCore.Slot()
    def _select_config_dir(self) -> None:
        def selected(path: pathlib.Path) -> None:
            self._book_combo_dirty = self._extract_builder.set_config_dir(path)
            self._guess_book_combo()
            self._refresh_from_state()

        _do_file_selection(self, QtWidgets.QFileDialog.FileMode.Directory, selected)

    @QtCore.Slot()
    def _select_input_pdf(self) -> None:
        def selected(path: pathlib.Path) -> None:
            self._extract_builder.input_pdf = path
            self._guess_book_combo()
            self._refresh_from_state()

        _do_file_selection(self, QtWidgets.QFileDialog.FileMode.ExistingFile, selected)

    def _guess_book_combo(self) -> None:
        cfg = self._extract_builder.cfg
        if cfg is None:
            return
        pdf = self._extract_builder.input_pdf
        if pdf is None:
            return
        filename = pdf.name
        for book_id, book in cfg.books.items():
            if filename == book.default_filename:
                self._extract_builder.book_id = book_id
                return

    @QtCore.Slot()
    def _select_book(self, index: int) -> None:
        if self._book_combo_dirty:
            return
        self._extract_builder.book_id = self._book_combo.itemData(index)
        self._refresh_from_state()

    @QtCore.Slot()
    def _select_output_dir(self) -> None:
        def selected(path: pathlib.Path) -> None:
            self._extract_builder.output_dir = path
            self._refresh_from_state()

        _do_file_selection(self, QtWidgets.QFileDialog.FileMode.Directory, selected)

    @QtCore.Slot()
    def _run_extraction(self) -> None:
        if self._extract is None:
            return
        if self._runner is not None:
            # Extraction already running.
            return
        self._runner = runnerwin.ExtractionRunnerWindow(
            self._extract,
            self._thread_pool,
            self._table_reader,
        )
        self._refresh_from_state()
        self._runner.closing.connect(self._runner_closing)
        self._runner.show()
        self._runner.start_extraction()

    @QtCore.Slot()
    def _runner_closing(self) -> None:
        self._runner = None
        self._refresh_from_state()


def _do_file_selection(
    parent: QtWidgets.QWidget,
    file_mode: QtWidgets.QFileDialog.FileMode,
    selected_callback: Callable[[pathlib.Path], None],
) -> None:
    @QtCore.Slot()
    def selected(path: str) -> None:
        selected_callback(pathlib.Path(path))

    dialog = QtWidgets.QFileDialog(parent=parent)
    dialog.setFileMode(file_mode)
    dialog.fileSelected.connect(selected)
    dialog.show()


def _bulk_enable(
    enabled: bool,
    *widgets: QtWidgets.QWidget,
) -> None:
    for widget in widgets:
        widget.setEnabled(enabled)


def _repopulate_book_combo(combo: QtWidgets.QComboBox, cfg: Optional[config.Config]) -> None:
    combo.clear()
    if cfg is None:
        return
    combo.addItem("<unselected>", None)
    for book_id, book in sorted(cfg.books.items(), key=lambda item: item[1].name):
        combo.addItem(book.name, book_id)


def _update_book_combo(combo: QtWidgets.QComboBox, book_id: Optional[str]) -> None:
    if book_id is None:
        combo.setCurrentIndex(0)
        return
    for i in range(combo.count()):
        if book_id == combo.itemData(i):
            combo.setCurrentIndex(i)
            return


def _update_path_label(label: QtWidgets.QLabel, path: Optional[pathlib.Path]) -> None:
    if path is None:
        label.setText("<not selected>")
    else:
        label.setText(str(path))


def _update_error_label(label: QtWidgets.QLabel, error: Optional[str]) -> None:
    if error is None:
        label.setText("")
    else:
        label.setText(error)
