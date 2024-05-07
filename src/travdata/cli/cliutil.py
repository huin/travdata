# -*- coding: utf-8 -*-
"""CLI utilities."""

import os


EX_CONFIG = getattr(os, "EX_CONFIG", 2)
EX_USAGE = getattr(os, "EX_USAGE", 3)


class CLIError(Exception):
    """Base class for CLI errors."""

    exit_code: int = 1


class UsageError(CLIError):
    """Exception for user usage error."""

    exit_code = getattr(os, "EX_USAGE", 2)
