" Vim syntax file
" Language: LWON (Light-Weight Object Notation)

if exists('b:current_syntax')
  finish
endif

" Long strings span lines, so highlighting is computed from the start of the file.
syntax sync fromstart

syntax match lwonDelimiter /[{}\[\]]/
syntax match lwonSeparator /[,:]/

" A short string extends to the next delimiter; keys are matched separately below.
syntax match lwonShortString /[^ \t{}\[\]:,"\\]\%([^{}\[\],\n\\]\|\\.\)*/

syntax match lwonInvalidEscape /\\./ contained
syntax match lwonEscape /\\\%(u{\x\{1,6}}\|u\x\{4}\|[rnfbt\\/"' }\]#:]\|$\)/ contained
" Escapes for surrogate code points are allowed only as a high surrogate
" followed by a low surrogate.
syntax match lwonInvalidEscape /\\u\%([dD][89a-fA-F]\x\x\|{0*[dD][89a-fA-F]\x\x}\)/ contained
syntax match lwonEscape /\\u[dD][89abAB]\x\x\\u[dD][c-fC-F]\x\x/ contained
syntax match lwonDoubledQuote /""/ contained
syntax region lwonString matchgroup=lwonQuote start=/"/ skip=/\\.\|""/ end=/"/
      \ contains=lwonComment,lwonEscape,lwonInvalidEscape,lwonDoubledQuote

" Keys begin a line or follow { or , and are followed by a colon, or by
" whitespace and then [, {, or ".
syntax match lwonKey /^\s*\zs\%([^ \t{}\[\]:",\\]\|\\.\)\%([^{}\[\]:"\\]\|\\.\)\{-}\ze\%(\s*:\|\s\+[\[{"]\)/
syntax match lwonKey /\%([{,]\s*\)\@<=\%([^ \t{}\[\]:",\\]\|\\.\)\%([^{}\[\]:"\\]\|\\.\)\{-}\ze\%(\s*:\|\s\+[\[{"]\)/
syntax match lwonQuotedKey /"\%([^"\\]\|\\.\|""\)*"\ze\s*:/

" A comment line begins with #; there are no comments within other lines.
syntax match lwonComment /^\s*#.*$/ contains=@Spell

highlight default link lwonComment Comment
highlight default link lwonString String
highlight default link lwonQuote String
highlight default link lwonShortString String
highlight default link lwonEscape SpecialChar
highlight default link lwonDoubledQuote SpecialChar
highlight default link lwonInvalidEscape Error
highlight default link lwonKey Label
highlight default link lwonQuotedKey Label
highlight default link lwonDelimiter Delimiter
highlight default link lwonSeparator Delimiter

let b:current_syntax = 'lwon'
