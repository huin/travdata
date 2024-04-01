# -*- coding: utf-8 -*-
"""QT utilities."""

# Pylint doesn't like QT much.
# pylint: disable=I1101

from PySide6 import QtWidgets


def make_group_vbox(
    title: str,
    *items: QtWidgets.QWidget | QtWidgets.QSpacerItem,
) -> QtWidgets.QGroupBox:
    """Helper to construct a group with vbox layout.

    :param title: Title of the group box widget.
    :param widgets: Widgets or spacing items to include in the group, in display
    order.
    :return: Group box.
    """
    vbox = QtWidgets.QVBoxLayout()
    for item in items:
        if isinstance(item, QtWidgets.QSpacerItem):
            vbox.addSpacerItem(item)
        else:
            vbox.addWidget(item)
    group_box = QtWidgets.QGroupBox(title)
    group_box.setLayout(vbox)
    return group_box
