/// Represents a unit test utility for managing test data, expected results, and error outputs.
///
/// This struct provides methods to construct paths for test case files, expected results,
/// and error results, as well as utilities to read test case files and compare results.
///
/// # Fields
/// - `test_data_folder`: The name of the subfolder containing the test data for this unit test.
///
/// # Methods
/// - `new(test_data_folder: &str) -> Self`: Creates a new `UnitTest` instance with the specified test data folder.
/// - `test_case_folder(&self) -> String`: Returns the absolute path to the test case folder.
/// - `expected_result_folder(&self) -> String`: Returns the relative path to the expected result folder.
/// - `result_error_folder(&self) -> String`: Returns the relative path to the result error folder.
/// - `get_test_case_file_paths(&self) -> io::Result<Vec<path::PathBuf>>`: Returns a sorted list of file paths in the test case folder.
/// - `check_result_with_file(&self, content: &str, file_name: &str)`: Compares the given content with the expected result file. If they differ or the file is missing, saves the content as an error result.
///
/// # Example
/// ```
/// let unit_test = UnitTest::new("my_test");
/// let test_cases = unit_test.get_test_case_file_paths().unwrap();
/// for case in test_cases {
///     // Run test and check result
/// }
/// ```

const MAIN_TEST_DATA_FOLDER: &str = "tests/test_data";
const TEST_CASE_FOLDER: &str = "test_case";
const EXPECTED_RESULT_FOLDER: &str = "expected_result";
const RESULT_ERROR_FOLDER: &str = "result_error";

use std::{fs, io, path};

pub struct UnitTest {
    test_data_folder: String,
}

impl UnitTest {
    pub fn new(test_data_folder: &str) -> Self {
        UnitTest {
            test_data_folder: test_data_folder.to_string(),
        }
    }

    pub fn test_case_folder(&self) -> String {
        let path = std::env::current_dir().unwrap();

        format!(
            "{}/{}/{}/{}",
            path.display(),
            MAIN_TEST_DATA_FOLDER,
            self.test_data_folder,
            TEST_CASE_FOLDER
        )
    }

    pub fn expected_result_folder(&self) -> String {
        format!(
            "{}/{}/{}",
            MAIN_TEST_DATA_FOLDER, self.test_data_folder, EXPECTED_RESULT_FOLDER
        )
    }

    pub fn result_error_folder(&self) -> String {
        format!(
            "{}/{}/{}",
            MAIN_TEST_DATA_FOLDER, self.test_data_folder, RESULT_ERROR_FOLDER
        )
    }

    pub fn get_test_case_file_paths(&self) -> io::Result<Vec<path::PathBuf>> {
        let mut entries = fs::read_dir(self.test_case_folder())?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        entries.sort();

        Ok(entries)
    }

    fn save_error_result(&self, content: &str, file_name: &str) {
        println!(
            "Saving error result to: {}/{}.result",
            self.result_error_folder(),
            file_name
        );
        std::fs::create_dir_all(self.result_error_folder())
            .expect("Failed to create result error folder");
        let complete_file_name = format!("{}/{}.result", self.result_error_folder(), file_name);
        std::fs::write(complete_file_name, content).expect("Failed to write to result error file");
    }

    pub fn check_result_with_file(&self, content: &str, file_name: &str) -> bool {
        let file_path = format!("{}/{}.result", self.expected_result_folder(), file_name);

        if let Ok(file_content) = std::fs::read_to_string(&file_path) {
            if content != file_content {
                println!("Content does not match the file: {}", file_path);
                self.save_error_result(content, file_name);
                false
            } else {
                true
            }
        } else {
            println!("Failed to read file: {}", file_path);
            self.save_error_result(content, file_name);
            false
        }
    }
}
