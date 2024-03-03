# Traveller Data Utils

Python library and assorted tools for assisting with the Mongoose Traveller
TTRPG system.

## Developing

The development environment is managed by [Poetry](https://python-poetry.org/).
Ensure that Poetry is installed.

Then optionally set Python virtual environment to in-project, which may help
your IDE find types more easily.

```shell
poetry config virtualenvs.in-project true
```

Then run the following to install dependencies required for development:

```shell
poetry install
```

To run tests:

```shell
poetry run pytest
```
