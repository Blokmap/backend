# Backend Setup Guide

## **Requirements**

Before you begin, make sure you have the following software installed:

- **Python 3.12** (check with `python --version` or `python3 --version`)
- **Poetry** (check with `poetry --version`)

## **Project Installation**

Install the dependencies:

```sh
poetry install
```

Activate the virtual environment (optional, Poetry does this automatically when using `poetry run`):

```sh
poetry shell
```

Make sure your IDE uses the correct Python interpreter to prevent IDE warnings. You can check its path with:

```sh
poetry env info --path
```

## **Starting the Local Development Server**

The development server configuration is part of the [deploy repository](). Head over there and read the instructions to start the development server.

## Useful Commands

Since we are working with a dockerized application, some commands need to be run inside the container. We provided a `Makefile` for these cases.

### **Database Migrations**

```sh
make migrate
```

### **Running the linter and formatter**

We use `ruff` for linting and formatting. You can run it with:

```sh
poetry run task lint
poetry run task format
```

### **Running the tests**

We use `pytest` for testing. You can run the tests with:

```sh
poetry run task test
```