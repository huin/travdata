# -*- coding: utf-8 -*-
"""CLI utilities."""

import os


class CLIError(Exception):
    """Base class for CLI errors."""

    exit_code: int = 1


class UsageError(CLIError):
    """Exception for user usage error."""

    exit_code = os.EX_USAGE
