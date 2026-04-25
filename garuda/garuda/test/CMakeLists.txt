find_package(GTest QUIET)

if(NOT GTest_FOUND)
    message(STATUS "GTest not found, tests will not be built")
    return()
endif()

include_directories(${CMAKE_SOURCE_DIR}/garuda/include)
include_directories(${LLVM_INCLUDE_DIRS})

set(GARUDA_TESTS
    Domain/DomainTest.cpp
    Group/GroupDomainTest.cpp
    Update/ProgressUpdateTest.cpp
)

add_executable(garuda-test ${GARUDA_TESTS})

target_link_libraries(garuda-test PRIVATE garuda)
target_link_libraries(garuda-test PRIVATE GTest::GTest)
target_link_libraries(garuda-test PRIVATE GTest::Main)
target_link_libraries(garuda-test PRIVATE ${LLVM_LIBRARIES})

add_test(NAME GarudaTests COMMAND garuda-test)