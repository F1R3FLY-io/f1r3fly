import XCTest
import SwiftTreeSitter
import TreeSitterRholang

final class TreeSitterRholangTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_rholang())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Rholang grammar")
    }
}
