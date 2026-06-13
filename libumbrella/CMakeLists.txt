cmake_minimum_required(VERSION 3.40)
project(umbrella VERSION 0.0.0)

set(CMAKE_CXX_STANDARD 26)

file(GLOB_RECURSE SRCS "src/*.cc")
add_library(umbrella_object OBJECT ${SRCS})

set_target_properties(umbrella_object PROPERTIES POSITION_INDEPENDENT_CODE ON)
target_include_directories(umbrella_object PUBLIC
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include/>
  $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)

add_library(umbrella_static STATIC $<TARGET_OBJECTS:umbrella_object>)
set_target_properties(umbrella_static PROPERTIES OUTPUT_NAME "umbrella")

add_library(umbrella_shared SHARED $<TARGET_OBJECTS:umbrella_object>)
set_target_properties(umbrella_shared PROPERTIES OUTPUT_NAME "umbrella")

add_executable(umbrella_exec "main.cc")
set_target_properties(umbrella_exec PROPERTIES OUTPUT_NAME "umbrella")
target_include_directories(umbrella_exec PUBLIC
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include/>
  $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)
target_link_libraries(umbrella_exec PUBLIC umbrella_shared)

include(GNUInstallDirs)

install(TARGETS umbrella_object umbrella_static umbrella_shared
  EXPORT umbrella_targets
  ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
  LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
  RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)

install(DIRECTORY "include/"
  DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
  FILES_MATCHING PATTERN "*.hpp"
)

install(EXPORT umbrella_targets
  FILE umbrellaTargets.cmake
  NAMESPACE umbrella::
  DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/umbrella
)

include(CMakePackageConfigHelpers)
write_basic_package_version_file(
  "${CMAKE_CURRENT_BINARY_DIR}/umbrellaConfigVersion.cmake"
  COMPATIBILITY SameMajorVersion
)

file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/umbrellaConfig.cmake"
  "include(\"\${CMAKE_CURRENT_LIST_DIR}/umbrellaTargets.cmake\")\n"
)

install(FILES
  "${CMAKE_CURRENT_BINARY_DIR}/umbrellaConfig.cmake"
  "${CMAKE_CURRENT_BINARY_DIR}/umbrellaConfigVersion.cmake"
  DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/umbrella
)