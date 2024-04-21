#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Builds configuration ZIP release asset."""

import argparse
import pathlib

from travdata import config


def main() -> None:
    """CLI entry point."""
    argparser = argparse.ArgumentParser(description=__doc__)
    argparser.add_argument("version")
    argparser.add_argument("config_dir", type=pathlib.Path)
    argparser.add_argument("config_zip", type=pathlib.Path)
    args = argparser.parse_args()

    config.create_config_zip(args.version, args.config_dir, args.config_zip)


if __name__ == "__main__":
    main()
