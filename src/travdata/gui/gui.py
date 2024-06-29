# -*- coding: utf-8 -*-
"""Entry point for GUI program to extract data from a PDF."""

# Pylint doesn't like QT much.
# pylint: disable=I1101

import sys

from PySide6 import QtCore, QtWidgets

from travdata import config
from travdata.extraction.pdf import cachingreader, tabulareader
from travdata.gui.extraction import cfgwin


def main() -> None:
    """Entry point for GUI program to extract data from a PDF."""
    app = QtWidgets.QApplication(sys.argv)
    app.setQuitOnLastWindowClosed(True)

    with (
        tabulareader.TabulaClient(
            force_subprocess=False,
        ) as tabula_client,
        cachingreader.optional_table_cache(
            delegate=tabula_client,
            disable=False,
        ) as table_reader,
    ):
        window = cfgwin.ExtractionConfigWindow(
            thread_pool=QtCore.QThreadPool(),
            table_reader=table_reader,
            default_config_path=config.get_default_config_path(),
        )

        window.show()
        sys.exit(app.exec())


if __name__ == "__main__":
    main()
