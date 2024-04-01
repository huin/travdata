# -*- coding: utf-8 -*-
"""Entry point for GUI program to extract data from a PDF."""

# Pylint doesn't like QT much.
# pylint: disable=I1101

import sys

from PySide6 import QtCore, QtWidgets

from travdata import config
from travdata.extraction import tabulautil
from travdata.gui.extraction import cfgwin


def main() -> None:
    """Entry point for GUI program to extract data from a PDF."""
    app = QtWidgets.QApplication(sys.argv)
    app.setQuitOnLastWindowClosed(True)

    with tabulautil.TabulaClient(
        force_subprocess=False,
    ) as tabula_client:
        window = cfgwin.ExtractionConfigWindow(
            thread_pool=QtCore.QThreadPool(),
            table_reader=tabula_client,
            config_dir=config.get_default_config_path(),
        )
        window.show()
        sys.exit(app.exec())


if __name__ == "__main__":
    main()
