# -*- coding: utf-8 -*-
"""QT utilities."""

# Pylint doesn't like QT much.
# pylint: disable=I1101

from typing import Iterable, Optional, TypeAlias

from PySide6 import QtWidgets


LayoutItem: TypeAlias = QtWidgets.QWidget | QtWidgets.QSpacerItem


_ERROR_STYLE = """
QLabel { color: red; }
"""


def set_error_style(*items: QtWidgets.QWidget) -> None:
    """Sets an error indicator style to the given widgets."""
    for item in items:
        item.setStyleSheet(_ERROR_STYLE)


def make_group_hbox(
    title: Optional[str],
    *items: LayoutItem,
) -> QtWidgets.QGroupBox:
    """Helper to construct a group with vbox layout.

    :param title: Title of the group box widget.
    :param widgets: Widgets or spacing items to include in the group, in display
    order.
    :return: Group box.
    """
    return _group_box(
        layout=QtWidgets.QHBoxLayout(),
        title=title,
        items=items,
    )


def make_group_vbox(
    title: Optional[str],
    *items: LayoutItem,
) -> QtWidgets.QGroupBox:
    """Helper to construct a group with vbox layout.

    :param title: Title of the group box widget.
    :param widgets: Widgets or spacing items to include in the group, in display
    order.
    :return: Group box.
    """
    return _group_box(
        layout=QtWidgets.QVBoxLayout(),
        title=title,
        items=items,
    )


def _group_box(
    layout: QtWidgets.QBoxLayout,
    title: Optional[str],
    items: Iterable[LayoutItem],
) -> QtWidgets.QGroupBox:
    for item in items:
        if isinstance(item, QtWidgets.QSpacerItem):
            layout.addSpacerItem(item)
        else:
            layout.addWidget(item)
    if title is None:
        group_box = QtWidgets.QGroupBox()
    else:
        group_box = QtWidgets.QGroupBox(title)
    group_box.setLayout(layout)
    return group_box
