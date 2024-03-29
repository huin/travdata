# -*- coding: utf-8 -*-
"""Pyinstaller hook to configure travdata when running via pyinstaller."""

from travdata import config

config.__executable_environment__ = "pyinstaller"
