# -*- coding: utf-8 -*-
"""Defines a window that configures extraction from a PDF."""

# Pylint doesn't like QT much.
# pylint: disable=I1101

import contextlib
import dataclasses
import pathlib
from typing import Callable, Optional

from PySide6 import QtCore, QtWidgets, QtGui

from travdata import commontext, config, filesio
from travdata.config import cfgerror
from travdata.extraction import bookextract, tableextract
from travdata.gui import qtutil
from travdata.gui.extraction import runnerwin


@dataclasses.dataclass
class _ExtractionConfigErrors:
    config_path: Optional[str] = None
    input_pdf: Optional[str] = None
    book_id: Optional[str] = None
    output_dir: Optional[str] = None


def _open_config_reader(
    config_type: filesio.IOType,
    config_path: pathlib.Path,
) -> contextlib.AbstractContextManager[filesio.Reader]:
    config_type = config_type.resolve_auto(config_path)
    return config_type.open(config_path)


@dataclasses.dataclass
class _ExtractionConfigBuilder:  # pylint: disable=too-many-instance-attributes
    _cfg: Optional[config.Config] = dataclasses.field(default=None, init=False)
    _cfg_error: Optional[str] = dataclasses.field(default=None, init=False)
    _cfg_version: Optional[str] = dataclasses.field(default=None, init=False)

    # Remaining fields enable building a config.ExtractionConfig.
    _config_type: filesio.IOType = dataclasses.field(default=filesio.IOType.AUTO, init=False)
    _config_path: Optional[pathlib.Path] = dataclasses.field(default=None, init=False)
    input_pdf: Optional[pathlib.Path] = None
    book_id: Optional[str] = None
    output_dir: Optional[pathlib.Path] = None

    @property
    def cfg(self) -> Optional[config.Config]:
        """Returns the current configuration."""
        return self._cfg

    @property
    def config_path(self) -> Optional[pathlib.Path]:
        """Returns the current configuration path."""
        return self._config_path

    @property
    def config_type(self) -> Optional[filesio.IOType]:
        """Returns the current configuration type."""
        return self._config_type

    @property
    def config_error(self) -> Optional[str]:
        """Returns the current configuration error."""
        return self._cfg_error

    @property
    def config_version(self) -> Optional[str]:
        """Returns the current configuration version."""
        return self._cfg_version

    def set_config_path(
        self,
        path: Optional[pathlib.Path],
    ) -> bool:
        """Sets the config dir, loading the config if set.

        :param path: New path to the configuration directory.
        :param force_update: Forces an update, even if it appears that there
        would be no change.
        :return: True if changed.
        """
        if self._config_path == path:
            return False

        self._config_path = path

        if self._config_path is None:
            self._config_type = filesio.IOType.AUTO
            self._cfg = None
        else:
            self._config_type = filesio.IOType.AUTO.resolve_auto(
                self._config_path,
            )
            with _open_config_reader(self._config_type, self._config_path) as cfg_reader:
                try:
                    cfg = config.load_config(cfg_reader)
                except filesio.NotFoundError as exc:
                    self._cfg = None
                    self._cfg_error = f"File not found in configuration: {exc}"
                    self._cfg_version = None
                except cfgerror.ConfigurationError as exc:
                    self._cfg = None
                    self._cfg_error = f"Configuration error: {exc}"
                    self._cfg_version = None
                else:
                    self._cfg = cfg
                    self._cfg_error = None
                    self._cfg_version = config.load_config_version(cfg_reader)

        if self._cfg is None or self.book_id not in self._cfg.books:
            self.book_id = None

        return True

    def set_config_type(self, config_type: filesio.IOType) -> bool:
        """Sets the config type.

        :param config_type: New config type.
        :return: True if changed.
        """
        if self._config_type == config_type:
            return False

        self.set_config_path(None)
        self._config_type = config_type
        return True

    def build_errors(self) -> _ExtractionConfigErrors:
        """Returns any errors in the builder (other than unspecified values)."""
        errors = _ExtractionConfigErrors()
        if self._cfg is None:
            errors.config_path = "Configuration must be selected."

        if self._cfg_error is not None:
            errors.config_path = self._cfg_error

        if self.input_pdf is not None:
            if not self.input_pdf.exists():
                errors.input_pdf = f"{self.input_pdf} does not exist."
            elif not self.input_pdf.is_file():
                errors.input_pdf = f"{self.input_pdf} is not a regular file."

        if self.output_dir is not None:
            if not self.output_dir.exists():
                errors.output_dir = f"{self.output_dir} does not exist."
            elif not self.output_dir.is_dir():
                errors.output_dir = f"{self.output_dir} is not a directory."

        return errors

    def build(self) -> Optional[bookextract.ExtractionConfig]:
        """Builds the extraction configuration, if complete."""
        if self._cfg is None:
            return None
        if self._config_path is None:
            return None
        if self.input_pdf is None:
            return None
        if self.book_id is None:
            return None
        if self.output_dir is None:
            return None

        return bookextract.ExtractionConfig(
            cfg_reader_ctx=_open_config_reader(self._config_type, self._config_path),
            out_writer_ctx=filesio.DirWriter.create(self.output_dir),
            input_pdf=self.input_pdf,
            book_id=self.book_id,
            overwrite_existing=False,
            with_tags=frozenset(),
            without_tags=frozenset(),
        )


class ExtractionConfigWindow(QtWidgets.QMainWindow):  # pylint: disable=too-many-instance-attributes
    """QT window to configure and start PDF extraction."""

    _default_config_path: Optional[pathlib.Path] = None

    # _extract_builder and _extract contain the data model, separately from any
    # widgets.
    _extract_builder: _ExtractionConfigBuilder
    _extract: Optional[bookextract.ExtractionConfig]

    _runner: Optional[runnerwin.ExtractionRunnerWindow]

    def __init__(
        self,
        thread_pool: QtCore.QThreadPool,
        table_reader: tableextract.TableReader,
        default_config_path: Optional[pathlib.Path],
    ) -> None:
        super().__init__()
        self.setWindowTitle("Travdata Extraction Setup")

        data_usage_text = QtWidgets.QLabel(commontext.DATA_USAGE)

        self._thread_pool = thread_pool
        self._table_reader = table_reader
        self._default_config_path = default_config_path

        self._runner = None

        self._extract_builder = _ExtractionConfigBuilder()
        self._extract_builder.set_config_path(default_config_path)
        self._extract = None

        self._config_path_label = QtWidgets.QLabel("")
        self._config_version_label = QtWidgets.QLabel("")
        self._config_version_label.setAlignment(QtCore.Qt.AlignmentFlag.AlignTrailing)
        self._config_path_error = QtWidgets.QLabel("")
        self._config_path_button = QtWidgets.QPushButton("Select configuration")
        self._config_path_button.clicked.connect(self._select_config_path)

        self._config_type_dir = QtWidgets.QRadioButton("Config directory")
        self._config_type_zip = QtWidgets.QRadioButton("Config ZIP")
        self._config_type_group = QtWidgets.QButtonGroup(self)
        self._config_type_group.addButton(
            self._config_type_dir,
            id=filesio.IOType.DIR.to_int_id(),
        )
        self._config_type_group.addButton(
            self._config_type_zip,
            id=filesio.IOType.ZIP.to_int_id(),
        )
        self._config_type_group.idToggled.connect(self._toggle_config_type)

        self._default_config_path_button = QtWidgets.QPushButton("Use default configuration")
        self._default_config_path_button.clicked.connect(self._select_default_config_path)

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

        qtutil.set_error_style(
            self._config_path_error,
            self._input_pdf_error,
            self._book_error,
            self._output_dir_error,
        )

        select_config_box = qtutil.make_group_hbox(
            None,
            self._config_path_button,
            self._default_config_path_button,
        )
        config_label_box = qtutil.make_group_hbox(
            None,
            self._config_path_label,
            self._config_version_label,
        )

        config_box = qtutil.make_group_vbox(
            "Extraction configuration",
            self._config_type_dir,
            self._config_type_zip,
            config_label_box,
            self._config_path_error,
            select_config_box,
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
        output_type = self._extract_builder.config_type
        _update_checked(self._config_type_dir, output_type == filesio.IOType.DIR)
        _update_checked(self._config_type_zip, output_type == filesio.IOType.ZIP)

        _bulk_enable(
            self._extract_builder.cfg is not None,
            self._input_pdf_button,
            self._book_combo,
            self._output_dir_button,
        )
        _update_path_label(self._config_path_label, self._extract_builder.config_path)
        if version := self._extract_builder.config_version:
            self._config_version_label.setText(f"Version: {version}")
        else:
            self._config_version_label.setText("Version: <unknown>")

        _update_path_label(self._input_pdf_label, self._extract_builder.input_pdf)
        if self._book_combo_dirty:
            _repopulate_book_combo(self._book_combo, self._extract_builder.cfg)
            self._book_combo_dirty = False
        _update_book_combo(self._book_combo, self._extract_builder.book_id)
        _update_path_label(self._output_dir_label, self._extract_builder.output_dir)

        errors = self._extract_builder.build_errors()
        _update_error_label(self._config_path_error, errors.config_path)
        _update_error_label(self._input_pdf_error, errors.input_pdf)
        _update_error_label(self._book_error, errors.book_id)
        _update_error_label(self._output_dir_error, errors.output_dir)

        self._extract = self._extract_builder.build()
        self._extract_button.setEnabled(self._extract is not None and self._runner is None)

    @QtCore.Slot()
    def _select_config_path(self) -> None:
        def selected(path: pathlib.Path) -> None:
            self._book_combo_dirty = self._extract_builder.set_config_path(path)
            self._guess_book_combo()
            self._refresh_from_state()

        match self._extract_builder.config_type:
            case filesio.IOType.DIR:
                file_mode = QtWidgets.QFileDialog.FileMode.Directory
                filter_ = "*"
            case filesio.IOType.ZIP:
                file_mode = QtWidgets.QFileDialog.FileMode.ExistingFile
                filter_ = "*.zip"
            case _:
                file_mode = QtWidgets.QFileDialog.FileMode.AnyFile
                filter_ = "*"

        _do_file_selection(
            parent=self,
            accept_mode=QtWidgets.QFileDialog.AcceptMode.AcceptOpen,
            file_mode=file_mode,
            selected_callback=selected,
            filter_=filter_,
        )

    @QtCore.Slot()
    def _select_default_config_path(self) -> None:
        self._extract_builder.set_config_path(self._default_config_path)
        self._refresh_from_state()

    @QtCore.Slot()
    def _toggle_config_type(self, id_: int, state: bool) -> None:
        if not state:
            return
        new_type = filesio.IOType.from_int_id(id_)
        if self._extract_builder.set_config_type(new_type):
            self._refresh_from_state()

    @QtCore.Slot()
    def _select_input_pdf(self) -> None:
        def selected(path: pathlib.Path) -> None:
            self._extract_builder.input_pdf = path
            self._guess_book_combo()
            self._refresh_from_state()

        _do_file_selection(
            parent=self,
            accept_mode=QtWidgets.QFileDialog.AcceptMode.AcceptOpen,
            file_mode=QtWidgets.QFileDialog.FileMode.ExistingFile,
            selected_callback=selected,
            filter_="*.pdf",
        )

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

        _do_file_selection(
            parent=self,
            accept_mode=QtWidgets.QFileDialog.AcceptMode.AcceptSave,
            file_mode=QtWidgets.QFileDialog.FileMode.Directory,
            selected_callback=selected,
            filter_="",
        )

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
    accept_mode: QtWidgets.QFileDialog.AcceptMode,
    file_mode: QtWidgets.QFileDialog.FileMode,
    selected_callback: Callable[[pathlib.Path], None],
    filter_: str,
) -> None:
    @QtCore.Slot()
    def selected(path: str) -> None:
        selected_callback(pathlib.Path(path))

    dialog = QtWidgets.QFileDialog(parent=parent, filter=filter_)
    dialog.setAcceptMode(accept_mode)
    dialog.setFileMode(file_mode)
    dialog.fileSelected.connect(selected)
    dialog.show()


def _update_checked(radio: QtWidgets.QRadioButton, state: bool) -> None:
    if radio.isChecked() != state:
        radio.setChecked(state)


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
