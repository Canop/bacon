cmake_minimum_required(VERSION 3.24)
project(example)

set(CMAKE_COLOR_DIAGNOSTICS on)

add_executable(example
  src/main.cpp
)
target_compile_options(example
  PUBLIC
    -Wall
)
