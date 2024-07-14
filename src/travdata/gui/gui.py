# -*- coding: utf-8 -*-
"""Entry point for GUI program to extract data from a PDF."""

# Pylint doesn't like QT much.
# pylint: disable=I1101

import multiprocessing
import sys

from PySide6 import QtCore, QtWidgets

from travdata import config
from travdata.gui.extraction import cfgwin


def main() -> None:
    """Entry point for GUI program to extract data from a PDF."""

    # Multiprocessing is used deeper in the program to perform the extraction,
    # so that Qt and threading does not create problems for STPyV8 and jpype
    # (such that they run in their own process and main thread - hence "spawn").
    #
    # freeze_support() is called here because many ways of running this program
    # call `main()`, rather than running this file as a script directly.
    multiprocessing.freeze_support()
    multiprocessing.set_start_method("spawn")

    app = QtWidgets.QApplication(sys.argv)
    app.setQuitOnLastWindowClosed(True)

    window = cfgwin.ExtractionConfigWindow(
        thread_pool=QtCore.QThreadPool(),
        default_config_path=config.get_default_config_path(),
    )

    window.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
