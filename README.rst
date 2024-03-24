Python library and assorted tools for assisting with the Mongoose
Traveller TTRPG system.

The extracted data is **not** for redistribution, as it is almost
certainly subject to copyright (I am not a lawyer - but it’s safer to
assume caution over distribution). This utility (and its output) is
intended as an aid to those who legally own a copy of the Mongoose
Traveller materials, and wish to make use of the data for their own
direct purposes. It is the sole responsibility of the user of this
program to use the extracted data in a manner that respects the
publisher’s IP rights.

.. important::

   Do not distribute the data extracted PDF files without explicit
   permission from the copyright holder.

The purpose of this tool is to extract the data for usage by the legal
owner of a copy of the original materal that it was extracted from.

.. _`_usage`:

Usage
=====

This package is primarily intended for the provided CLI tools, but API
access is also possible.

For any usage of the CLI or API that involves extracting CSV data from
PDFs, the Java Runtime Environment (JRE) must be installed on the
system.

.. _`_cli_literal_travdata_cli_extractcsvtables_literal`:

CLI ``travdata_cli extractcsvtables``
-------------------------------------

This tool extracts CSV files from tables in the given PDF, based on the
given configuration files that specifies the specifics of how those
tables can be turned into useful CSV data. As such, it only supports
extraction of tables from known PDF files, where the individual tables
have been configured.

The general form of the command is:

.. code:: shell

   travdata_cli extractcsvtables -c CONFIG_DIR BOOK_NAME INPUT.PDF OUT_DIR

Where:

``CONFIG_DIR``
   is the path to the directory containing a ``config.yaml`` file, and
   subdirectories and ``*.tabula-template.json`` files. This contains
   information guiding the extraction, and is specific to the PDF being
   read from. These configurations are provided with the source code to
   this program in the directories under the ``config`` directory.

``BOOK_NAME``
   is the identifier for the book to extract tables from. This selects
   the correct book’s configuration from the ``CONFIG_DIR``. Use
   ``travdata_cli -c CONFIG_DIR listbooks`` to list accepted values for
   this argument.

``INPUT.PDF``
   is the path to the PDF file to read tables from.

``OUT_DIR``
   is the path to a (potentially not existing) directory to output the
   resulting CSV files. This will result in containing a directory and
   file structure that mirrors that in ``CONFIG_DIR``, but will contain
   ``.csv`` rather than ``.tabula-template.json`` files.

At the present time, the only supported input PDF file is the Mongoose
Traveller Core Rulebook 2022, and not all tables are yet supported for
extraction.

Example:

.. code:: shell

   travdata_cli extractcsvtables -c path/to/config \
       core_rulebook_2022 path/to/update_2022_core_rulebook.pdf \
       path_to_output_dir

.. _`_developing`:

Developing
==========

See
```development.adoc`` <https://github.com/huin/travdata/blob/main/development.adoc>`__
for more information on developing and adding more tables to the
configuration.
