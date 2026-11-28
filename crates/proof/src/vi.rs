//! Telling Vietnamese text apart from English, so a document typed in Vietnamese under the default
//! English proofing language isn't covered in red underlines (as Word's "detect language
//! automatically" does). There are no Vietnamese proofing tools yet, so Vietnamese text is treated
//! like any language without them: no spelling or grammar marks.
//!
//! The test is orthographic. Vietnamese writes most syllables with letters no other Latin-script
//! language shares: the horn letters (`ơ ư`), `ĩ ũ`, and the tone-marked vowels of the Latin
//! Extended Additional block (`ạ ả ấ ầ … ỹ`), precomposed or as combining hook above, dot below
//! and horn. French and Portuguese share `â ê ô à é`, Romanian `ă` and Croatian, Serbian and
//! Bosnian `đ`, so those alone don't count.

/// Does `c` occur in Vietnamese words but (practically) nowhere else?
fn vietnamese_letter(c: char) -> bool {
    matches!(c, 'ơ' | 'Ơ' | 'ư' | 'Ư' | 'ĩ' | 'Ĩ' | 'ũ' | 'Ũ' | '\u{1EA0}'..='\u{1EF9}' | '\u{0309}' | '\u{0323}' | '\u{031B}')
}

/// Is `word` written with a Vietnamese-only letter (`Việt`, `ơn`)? Such a word is never an
/// English misspelling.
pub fn is_vietnamese_word(word: &str) -> bool {
    let combining = |c: char| matches!(c, '\u{0300}'..='\u{036F}');
    // Two marks stacked on one letter (decomposed `ế` = e + circumflex + acute) are Vietnamese too.
    word.chars().any(vietnamese_letter) || word.chars().zip(word.chars().skip(1)).any(|(a, b)| combining(a) && combining(b))
}
