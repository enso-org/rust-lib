#![deny(unconditional_recursion)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_import_braces)]

//! This file contains the code defining a lexer for the following small language. Due to the way in
//! which the code-generation from the flexer is used, it has to be defined in a separate crate from
//! the site at which it's used. For the actual tests of this code, please see
//! `flexer-testing/generation`.
//!
//! The language here is being defined as follows:
//!
//! a-word      = 'a'+;
//! b-word      = 'b'+;
//! word        = a-word | b-word;
//! space       = ' ';
//! spaced-word = space, word;
//! language    = word, spaced-word*;
//!
//! Please note that there is a fair amount of duplicated code between this test and the
//! `lexer_generated_api_test` file. This is to present the full view of what each portion of the
//! process looks like.

use enso_flexer::*;
use enso_flexer::prelude::*;

use enso_flexer;
use enso_flexer::automata::pattern::Pattern;
use enso_flexer::group::Registry;
use enso_flexer::prelude::logger::Disabled;
use enso_flexer::prelude::reader::BookmarkManager;



// ====================
// === Type Aliases ===
// ====================

type Logger = Disabled;



// ===========
// === AST ===
// ===========

/// A very simple AST, sufficient for the simple language being defined.
#[derive(Clone,Debug,PartialEq)]
pub enum Token {
    /// A word from the input, consisting of a sequence of all `a` or all `b`.
    Word(String),
    /// A token that the lexer is unable to recognise.
    Unrecognized(String),
}
impl Token {
    /// Construct a new word token.
    pub fn word(name:impl Into<String>) -> Token {
        Token::Word(name.into())
    }

    /// Construct a new unrecognized token.
    pub fn unrecognized(name:impl Into<String>) -> Token {
        Token::Unrecognized(name.into())
    }
}

/// A representation of a stream of tokens.
#[allow(missing_docs)]
#[derive(Clone,Debug,Default,PartialEq)]
pub struct TokenStream {
    tokens:Vec<Token>
}

impl TokenStream {
    /// Append the provided token to the token stream.
    pub fn push(&mut self,token:Token) {
        self.tokens.push(token);
    }
}


// === Trait Impls ===

impl From<Vec<Token>> for TokenStream {
    fn from(tokens: Vec<Token>) -> Self {
        TokenStream {tokens}
    }
}



// ==================
// === Test Lexer ===
// ==================

/// The definition of a test lexer for the above-described language.
#[derive(Debug)]
pub struct TestLexer {
    lexer:Flexer<TestState,TokenStream,Logger>
}

impl Deref for TestLexer {
    type Target = Flexer<TestState,TokenStream,Logger>;
    fn deref(&self) -> &Self::Target {
        &self.lexer
    }
}

impl DerefMut for TestLexer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lexer
    }
}

impl TestLexer {
    /// Creates a new instance of this lexer.
    pub fn new() -> Self {
        let logger = Logger::new("TestLexer");
        let lexer  = Flexer::new(logger);
        TestLexer{lexer}
    }
}

/// Rules for the root state.
#[allow(dead_code,missing_docs)]
impl TestLexer {
    fn on_first_word<R:LazyReader>(&mut self, _reader:&mut R) {
        let str = self.current_match.clone();
        let ast = Token::Word(str);
        self.output.push(ast);
        let id = self.seen_first_word_state;
        self.push_state(id);
    }

    fn on_err_suffix_first_word<R:LazyReader>(&mut self, _reader:&mut R) {
        let ast = Token::Unrecognized(self.current_match.clone());
        self.output.push(ast);
    }

    fn on_no_err_suffix_first_word<R:LazyReader>(&mut self, _reader:&mut R) {}

    fn rules_in_root(lexer:&mut TestLexer) {
        let a_word        = Pattern::char('a').many1();
        let b_word        = Pattern::char('b').many1();
        let any           = Pattern::any();
        let end           = Pattern::eof();

        let root_group_id = lexer.initial_state;
        let root_group    = lexer.groups_mut().group_mut(root_group_id);

        root_group.create_rule(&a_word,"self.on_first_word(reader)");
        root_group.create_rule(&b_word,"self.on_first_word(reader)");
        root_group.create_rule(&end,   "self.on_no_err_suffix_first_word(reader)");
        root_group.create_rule(&any,   "self.on_err_suffix_first_word(reader)");
    }
}

/// Rules for the "seen first word" state.
#[allow(dead_code,missing_docs)]
impl TestLexer {
    fn on_spaced_word<R:LazyReader>(&mut self, _reader:&mut R) {
        let str = self.current_match.clone();
        let ast = Token::Word(String::from(str.trim()));
        self.output.push(ast);
    }

    fn on_err_suffix<R:LazyReader>(&mut self, reader:&mut R) {
        self.on_err_suffix_first_word(reader);
        self.pop_state();
    }

    fn on_no_err_suffix<R:LazyReader>(&mut self, reader:&mut R) {
        self.on_no_err_suffix_first_word(reader);
        self.pop_state();
    }

    fn rules_in_seen_first_word(lexer:&mut TestLexer) {
        let a_word        = Pattern::char('a').many1();
        let b_word        = Pattern::char('b').many1();
        let space         = Pattern::char(' ');
        let spaced_a_word = &space >> &a_word;
        let spaced_b_word = &space >> &b_word;
        let any           = Pattern::any();
        let end           = Pattern::eof();

        let seen_first_word_group_id = lexer.seen_first_word_state;
        let seen_first_word_group    = lexer.groups_mut().group_mut(seen_first_word_group_id);

        seen_first_word_group.create_rule(&spaced_a_word,"self.on_spaced_word(reader)");
        seen_first_word_group.create_rule(&spaced_b_word,"self.on_spaced_word(reader)");
        seen_first_word_group.create_rule(&end,          "self.on_no_err_suffix(reader)");
        seen_first_word_group.create_rule(&any,          "self.on_err_suffix(reader)");
    }
}


// === Trait Impls ===

impl enso_flexer::Definition for TestLexer {
    fn define() -> Self {
        let mut lexer = TestLexer::new();

        TestLexer::rules_in_seen_first_word(&mut lexer);
        TestLexer::rules_in_root(&mut lexer);

        lexer
    }

    fn groups(&self) -> &Registry {
        self.lexer.groups()
    }

    fn set_up(&mut self) {}

    fn tear_down(&mut self) {}
}

impl Default for TestLexer {
    fn default() -> Self {
        TestLexer::new()
    }
}



// ===================
// === Lexer State ===
// ===================

/// The stateful components of the test lexer.
#[derive(Debug)]
pub struct TestState {
    /// The registry for groups in the lexer.
    lexer_states:group::Registry,
    /// The initial state of the lexer.
    initial_state:group::Identifier,
    /// The state entered when the first word has been seen.
    seen_first_word_state:group::Identifier,
    /// The bookmarks for this lexer.
    bookmarks:BookmarkManager
}


// === Trait Impls ===

impl enso_flexer::State for TestState {
    fn new(_logger:&impl AnyLogger) -> Self {
        let mut lexer_states      = group::Registry::default();
        let initial_state         = lexer_states.define_group("ROOT",None);
        let seen_first_word_state = lexer_states.define_group("SEEN FIRST WORD",None);
        let bookmarks             = BookmarkManager::new();
        Self{lexer_states,initial_state,seen_first_word_state,bookmarks}
    }

    fn initial_state(&self) -> group::Identifier {
        self.initial_state
    }

    fn groups(&self) -> &group::Registry {
        &self.lexer_states
    }

    fn groups_mut(&mut self) -> &mut group::Registry {
        &mut self.lexer_states
    }

    fn bookmarks(&self) -> &BookmarkManager {
        &self.bookmarks
    }

    fn bookmarks_mut(&mut self) -> &mut BookmarkManager {
        &mut self.bookmarks
    }

    fn specialize(&self) -> Result<String,GenError> {
        generate::specialize(self,"TestLexer","TokenStream")
    }
}
# [allow (missing_docs , dead_code , clippy :: all)] impl TestLexer { pub fn run < R : LazyReader > (& mut self , mut reader : R) -> LexingResult < TokenStream > { self . set_up () ; reader . advance_char (& mut self . bookmarks) ; while self . run_current_state (& mut reader) == StageStatus :: ExitSuccess { } let result = match self . status { StageStatus :: ExitFinished => LexingResult :: success (mem :: take (& mut self . output)) , StageStatus :: ExitFail => LexingResult :: failure (mem :: take (& mut self . output)) , _ => LexingResult :: partial (mem :: take (& mut self . output)) } ; self . tear_down () ; result } fn run_current_state < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { self . status = StageStatus :: Initial ; let mut finished = false ; while let Some (next_state) = self . status . continue_as () { self . logger . debug (| | format ! ("Current character is {:?}." , reader . character () . char)) ; self . logger . debug (| | format ! ("Continuing in {:?}." , next_state)) ; self . status = self . step (next_state , reader) ; if finished && reader . finished (self . bookmarks ()) { self . logger . info ("Input finished.") ; self . status = StageStatus :: ExitFinished } finished = reader . character () . is_eof () ; if self . status . should_continue () { match reader . character () . char { Ok (char) => { reader . append_result (char) ; self . logger . info (| | format ! ("Result is {:?}." , reader . result ())) ; } , Err (enso_flexer :: prelude :: reader :: Error :: EOF) => { self . logger . info ("Reached EOF.") ; } , Err (enso_flexer :: prelude :: reader :: Error :: EndOfGroup) => { let current_state = self . current_state () ; let group_name = self . groups () . group (current_state) . name . as_str () ; let err = format ! ("Missing rules for state {}." , group_name) ; self . logger . error (err . as_str ()) ; panic ! (err) } Err (_) => { self . logger . error ("Unexpected error!") ; panic ! ("Unexpected error!") } } reader . advance_char (& mut self . bookmarks) ; } } self . status } fn step < R : LazyReader > (& mut self , next_state : SubStateId , reader : & mut R) -> StageStatus { let current_state : usize = self . current_state () . into () ; match current_state { 0 => self . dispatch_in_state_0 (next_state , reader) , 1 => self . dispatch_in_state_1 (next_state , reader) , _ => unreachable_panic ! ("Unreachable state reached in lexer.") , } } fn state_0_to_0 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { 0 ..= 96 => { StageStatus :: ContinueWith (1 . into ()) } , 97 => { StageStatus :: ContinueWith (2 . into ()) } , 98 => { StageStatus :: ContinueWith (3 . into ()) } , 99 ..= 18446744073709551614 => { StageStatus :: ContinueWith (1 . into ()) } , _ => { StageStatus :: ContinueWith (4 . into ()) } , } } fn state_0_to_1 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_0_rule_3 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn state_0_to_2 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { 0 ..= 96 => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_0_rule_0 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , 97 => { StageStatus :: ContinueWith (5 . into ()) } , _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_0_rule_0 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn state_0_to_3 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { 0 ..= 97 => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_0_rule_1 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , 98 => { StageStatus :: ContinueWith (6 . into ()) } , _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_0_rule_1 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn state_0_to_4 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_0_rule_2 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn state_0_to_5 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { 0 ..= 96 => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_0_rule_0 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , 97 => { StageStatus :: ContinueWith (5 . into ()) } , _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_0_rule_0 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn state_0_to_6 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { 0 ..= 97 => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_0_rule_1 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , 98 => { StageStatus :: ContinueWith (6 . into ()) } , _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_0_rule_1 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn dispatch_in_state_0 < R : LazyReader > (& mut self , new_state_index : SubStateId , reader : & mut R) -> StageStatus { match new_state_index . into () { 0 => self . state_0_to_0 (reader) , 1 => self . state_0_to_1 (reader) , 2 => self . state_0_to_2 (reader) , 3 => self . state_0_to_3 (reader) , 4 => self . state_0_to_4 (reader) , 5 => self . state_0_to_5 (reader) , 6 => self . state_0_to_6 (reader) , _ => unreachable_panic ! ("Unreachable state reached in lexer.") } } fn group_0_rule_0 < R : LazyReader > (& mut self , reader : & mut R) { self . on_first_word (reader) } fn group_0_rule_1 < R : LazyReader > (& mut self , reader : & mut R) { self . on_first_word (reader) } fn group_0_rule_2 < R : LazyReader > (& mut self , reader : & mut R) { self . on_no_err_suffix_first_word (reader) } fn group_0_rule_3 < R : LazyReader > (& mut self , reader : & mut R) { self . on_err_suffix_first_word (reader) } fn state_1_to_0 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { 0 ..= 31 => { StageStatus :: ContinueWith (1 . into ()) } , 32 => { StageStatus :: ContinueWith (2 . into ()) } , 33 ..= 18446744073709551614 => { StageStatus :: ContinueWith (1 . into ()) } , _ => { StageStatus :: ContinueWith (3 . into ()) } , } } fn state_1_to_1 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_1_rule_3 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn state_1_to_2 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { 0 ..= 96 => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_1_rule_3 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , 97 => { StageStatus :: ContinueWith (4 . into ()) } , 98 => { StageStatus :: ContinueWith (5 . into ()) } , _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_1_rule_3 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn state_1_to_3 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_1_rule_2 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn state_1_to_4 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { 0 ..= 96 => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_1_rule_0 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , 97 => { StageStatus :: ContinueWith (6 . into ()) } , _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_1_rule_0 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn state_1_to_5 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { 0 ..= 97 => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_1_rule_1 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , 98 => { StageStatus :: ContinueWith (7 . into ()) } , _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_1_rule_1 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn state_1_to_6 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { 0 ..= 96 => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_1_rule_0 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , 97 => { StageStatus :: ContinueWith (6 . into ()) } , _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_1_rule_0 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn state_1_to_7 < R : LazyReader > (& mut self , reader : & mut R) -> StageStatus { match u64 :: from (reader . character ()) { 0 ..= 97 => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_1_rule_1 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , 98 => { StageStatus :: ContinueWith (7 . into ()) } , _ => { let matched_bookmark = self . bookmarks . matched_bookmark ; self . current_match = reader . pop_result () ; self . group_1_rule_1 (reader) ; self . bookmarks . bookmark (matched_bookmark , reader) ; StageStatus :: ExitSuccess } , } } fn dispatch_in_state_1 < R : LazyReader > (& mut self , new_state_index : SubStateId , reader : & mut R) -> StageStatus { match new_state_index . into () { 0 => self . state_1_to_0 (reader) , 1 => self . state_1_to_1 (reader) , 2 => self . state_1_to_2 (reader) , 3 => self . state_1_to_3 (reader) , 4 => self . state_1_to_4 (reader) , 5 => self . state_1_to_5 (reader) , 6 => self . state_1_to_6 (reader) , 7 => self . state_1_to_7 (reader) , _ => unreachable_panic ! ("Unreachable state reached in lexer.") } } fn group_1_rule_0 < R : LazyReader > (& mut self , reader : & mut R) { self . on_spaced_word (reader) } fn group_1_rule_1 < R : LazyReader > (& mut self , reader : & mut R) { self . on_spaced_word (reader) } fn group_1_rule_2 < R : LazyReader > (& mut self , reader : & mut R) { self . on_no_err_suffix (reader) } fn group_1_rule_3 < R : LazyReader > (& mut self , reader : & mut R) { self . on_err_suffix (reader) } }