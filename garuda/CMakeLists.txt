cmake_minimum_required(VERSION 3.20)
project(Garuda LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

include_directories(${CMAKE_SOURCE_DIR}/garuda/include)

find_package(LLVM REQUIRED CONFIG)
message(STATUS "Found LLVM ${LLVM_PACKAGE_VERSION}")
message(STATUS "Using LLVMConfig.cmake in: ${LLVM_DIR}")

include_directories(${LLVM_INCLUDE_DIRS})
add_definitions(${LLVM_DEFINITIONS})

set(GARUDA_SOURCES
    garuda/src/Domain/AbstractDomain.cpp
    garuda/src/Domain/ValueDomain.cpp
    garuda/src/Domain/IntegerDomain.cpp
    garuda/src/Domain/BooleanDomain.cpp
    garuda/src/Group/GroupDomain.cpp
    garuda/src/Group/GroupDomainManager.cpp
)

add_library(garuda STATIC ${GARUDA_SOURCES})

target_link_libraries(garuda PRIVATE ${LLVM_LIBRARIES})
target_link_libraries(garuda PRIVATE LLVM-22)

set_target_properties(garuda PROPERTIES
    LINKER_LANGUAGE CXX
    POSITION_INDEPENDENT_CODE ON
)

add_executable(garuda-tool garuda/tool/main.cpp)
target_link_libraries(garuda-tool PRIVATE garuda)

enable_testing()