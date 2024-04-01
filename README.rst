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

.. _`_reporting_issues`:

Reporting issues
================

Report any problems you encounter or feature requests to
https://github.com/huin/travdata/issues.

Please include:

-  information about which operating system you are using the program
   on,

-  steps to reproduce the problem,

-  what you expected to happen,

-  what actually happened.

Ideally include text output of any error messages from the program,
and/or screenshots to demonstrate the problem if text output is not
relevant.

.. _`_usage`:

Usage
=====

This package is primarily intended for the provided CLI tools, but API
access is also possible.

.. _`_requirements`:

Requirements
============

Java Runtime Environment (JRE) must be installed. This is required by
the code that extracts tables from PDFs. If not already installed, get
it from `java.com <https://www.java.com/en/download/>`__.

.. _`_installation`:

Installation
------------

.. _`_prebuilt`:

Prebuilt
~~~~~~~~

You can download an executable version of the application for your
platform at
`github.com/huin/travdata/releases <https://github.com/huin/travdata/releases>`__.
Currently executables are only generated for Linux and Windows, and seem
to work on the author’s machines. A MacOS binary is also released, but
it has not been tested.

Once downloaded, extract the ``.zip`` file to a suitable location. You
can most easily use the command line interface from the directory that
it was unpacked to.

You may also wish to make a shortcut to the ``travdata_gui`` executable,
but it can also be run directly from the unzipped directory.

.. _`_pip_install`:

Pip install
~~~~~~~~~~~

This may work on platforms that have no prebuilt executable. Assuming
that you have Python 3.11 or later installed, and you are running
something similar to Linux, perform the following commands to install
into a Python virtual environment:

.. code:: console

   mkdir travdata
   cd travdata
   python -m venv venv
   source ./venv/bin/activate

You will also need to download a copy of the source code, in order to
get a copy of the configuration. Visit
`releases <https://github.com/huin/travdata/releases>`__, pick a recent
release, and download the "Source code" zip file. Extract the ``config``
directory from it, and place it in the ``travdata`` directory you
created earlier, such that the ``travdata`` directory contains two
subdirectories:

-  ``config``

-  ``venv``

At this point, you can run ``python -m travdata.cli.cli`` instead of
running ``travdata_cli`` from other examples.

.. _`_gui_literal_travdata_gui_literal`:

GUI ``travdata_gui``
--------------------

The GUI binary provides a user interface to aid in extracting CSV files,
similarly to ``travdata_cli extractcsvtables``.

.. figure::
   https://raw.githubusercontent.com/huin/travdata/main/images/extraction_gui.png
   :alt: Screenshot of extraction configuration GUI

1. "Extraction configuration" selects the configuration for extraction
   from PDFs. It should detect its own configuration automatically. If
   it does not, click "Select configuration" and select the
   ``_internal/config`` directory that should have come with the
   download of the program.

2. "Input PDF" selects the PDF to extract from, and the book that that
   PDF corresponds to.

   Note: only a very limited number of books are supported - at the time
   of writing, only the Core Rulebook 2022 update.

   Click "Select PDF" and choose the input PDF file.

3. If selecting a PDF file did not automatically choose the correct book
   (based on its filename), choose it from the drop-down box below
   "Select PDF".

4. "Output directory" selects a directory to write the extracted CSV
   data into. Choose an empty directory.

5. "Extract" should now be enabled. Click it to start extraction. It
   will open a window to display progress and any errors.

6. Close the extraction window once extraction is completed (and if you
   no longer need the output).

Note: the program will not extract the same table again if it sees that
the CSV file is present in the output directory. If you wish to force
re-extraction, delete some or all files from the output directory.

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

   travdata_cli extractcsvtables BOOK_NAME INPUT.PDF OUT_DIR

Where:

``BOOK_NAME``
   is the identifier for the book to extract tables from. This selects
   the correct book’s configuration from the files that . Use
   ``travdata_cli listbooks`` to list accepted values for this argument.

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

   travdata_cli extractcsvtables \
       core_rulebook_2022 path/to/update_2022_core_rulebook.pdf \
       path_to_output_dir

.. _`_developing`:

Developing
==========

See
```development.adoc`` <https://github.com/huin/travdata/blob/main/development.adoc>`__
for more information on developing and adding more tables to the
configuration.
