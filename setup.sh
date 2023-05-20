#!/bin/bash -i

terminalWidth=$(tput cols)
if [ -z "$terminalWidth" ]; then
    terminalWidth=80
fi

function printGreen() {
    printf "\e[32m$1\e[0m\n"
}

function printRed() {
    printf "\e[31m$1\e[0m\n"
}

function printYellow() {
    printf "\e[33m$1\e[0m\n"
}

function centerText() {
    textLength=${#1}
    spaces=$((terminalWidth-textLength-2))
    leftSpaces=$((spaces/2))
    rightSpaces=$((spaces-leftSpaces+2))

    printf "\e[36m│\e[0m"
    for ((i=1; i < $leftSpaces; i++)); do
        printf " "
    done
    if [ "$3" = true ]; then
        printf "\e[1;${2}m$1\e[0m"
    else
        printf "\e[${2}m$1\e[0m"
    fi
    for ((i=1; i < $rightSpaces; i++)); do
        printf " "
    done
    printf "\e[36m│\e[0m\n"
}

function spaces() {
    printf "\e[36m│\e[0m" 
    for ((j=1; j < $terminalWidth - 1; j++)); do
        printf " "
    done
    printf "\e[36m│\e[0m\n"
}

function topBorder() {
    printf "\e[36m┌"
    for ((i=1; i < $terminalWidth - 1; i++)); do
        printf "─"
    done
    printf "┐\e[0m\n"
}

function bottomBorder() {
    printf "\e[36m└"
    for ((i=1; i < $terminalWidth - 1; i++)); do
        printf "─"
    done
    printf "┘\e[0m\n"
}

function ubuntuOrDebian() {
    if [[ "$(cat /etc/*-release | grep ^ID=)" == "ID=ubuntu" || "$(cat /etc/*-release | grep ^ID=)" == "ID=debian" ]]; then
        return 0
    else
        return 1
    fi
}

clear

topBorder
spaces
spaces
spaces
centerText "Home Api Rust" "32" true
spaces
centerText "Welcome to the setup script for the project" "95"
spaces
spaces
spaces
bottomBorder

printGreen "Building project"

cargo build --release

printYellow "Generating api-key secret"

SECRET=$(python3 -c "import os; import binascii; print(binascii.hexlify(os.urandom(32)))")
ALGORITHM="HS256"
substring=$(echo "$SECRET" | sed "s/'//g" | awk '{print $1}' | cut -c 2-)

if [[ -f .env ]]; then
    printGreen ".env exists, skipping."
else
    printYellow ".env does not exist, creating it."
    echo "secret=$substring" > .env
    echo "algorithm=$ALGORITHM" >> .env
    echo "port=8000" >> .env
fi

printGreen "api-key secret is generated."