# -*- coding: utf-8 -*-
"""CLI utilities."""

import os


EX_CONFIG = getattr(os, "EX_CONFIG", 78)
EX_SOFTWARE = getattr(os, "EX_SOFTWARE", 70)
EX_USAGE = getattr(os, "EX_USAGE", 64)


class CLIError(Exception):
    """Base class for CLI errors."""

    exit_code: int = 1


class UsageError(CLIError):
    """Exception for user usage error."""

    exit_code = EX_USAGE
