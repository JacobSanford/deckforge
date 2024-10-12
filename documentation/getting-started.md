## Getting Started
This application is developed in Linux, on Ubuntu. That is currently the only supported OS for this application.

## Prerequisites

```
sudo apt-get update
sudo apt-get install libssl-dev libbz2-dev libreadline-dev libsqlite3-dev liblzma-dev tk-dev ffmpeg libmagickwand-dev
```

## pyenv
First, to manage different python versions we install pyenv.

### Install


```
curl https://pyenv.run | bash
```

### Configure Shell

https://github.com/pyenv/pyenv#installation

### Set a global version

```
pyenv install 3.10
pyenv global 3.10
```

## poetry

### Install

```
curl -sSL https://install.python-poetry.org | python -
```

### Add path

```
fish_add_path /home/jsanford/.local/bin
```

### Config venvs in path (vscode, etc)
```
poetry config virtualenvs.in-project true
```

## Install the app

```
poetry install
```

If you've already set up the env, you must remove it and reinstall

```
poetry env list  # shows the name of the current environment
poetry env remove <current environment>
poetry install  # will create a new environment using your updated configuration
```
