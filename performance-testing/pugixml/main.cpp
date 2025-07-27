#include "pugixml.hpp"
#include <iostream>

#include <chrono>
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>

std::string readFileToString(const std::string &filePath) {
  std::ifstream file(filePath, std::ios::binary);
  if (!file.is_open()) {
    return "";
  }

  return std::string(std::istreambuf_iterator<char>(file), std::istreambuf_iterator<char>());
}

int main() {
  pugi::xml_document doc;

  auto content = readFileToString("large.xhtml");

  auto start = std::chrono::system_clock::now();

  pugi::xml_parse_result result = doc.load(content.c_str());

  auto end     = std::chrono::system_clock::now();
  auto elapsed = end - start;
  std::cout << elapsed.count() << '\n';

  //   if (result) {
  //     std::cout << "XML parsed successfully!" << std::endl;
  //   } else {
  //     std::cerr << "XML parsing failed: " << result.description() << std::endl;
  //   }

  return 0;
}
// Compile with: g++ main.cpp pugixml.cpp -o pugixml-test