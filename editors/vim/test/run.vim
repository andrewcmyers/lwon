" Check the syntax highlighting of ../vscode/test/syntax.test.lwon.
" Usage, from editors/vim:
"   vim -Nu NONE -es -c 'set rtp^=.' -c 'filetype plugin on' -c 'syntax on'
"       -S test/run.vim
" Prints any failures and exits with a nonzero status if there are any.

edit ../vscode/test/syntax.test.lwon
if &filetype !=# 'lwon'
  call writefile(['filetype not detected: ' . &filetype], '/dev/stderr')
  cquit
endif

" [line, column, expected syntax group]
let s:checks = [
      \ [3, 1, 'lwonComment'],
      \ [7, 1, 'lwonKey'], [7, 5, 'lwonSeparator'], [7, 7, 'lwonShortString'],
      \ [12, 1, 'lwonKey'], [12, 9, 'lwonKey'], [12, 10, 'lwonSeparator'],
      \ [15, 1, 'lwonQuotedKey'], [15, 6, 'lwonSeparator'], [15, 8, 'lwonShortString'],
      \ [20, 1, 'lwonDelimiter'], [20, 2, 'lwonKey'], [20, 5, 'lwonDelimiter'],
      \ [20, 6, 'lwonShortString'], [20, 7, 'lwonSeparator'], [20, 13, 'lwonKey'],
      \ [20, 16, 'lwonQuote'], [20, 17, 'lwonString'], [20, 18, 'lwonDoubledQuote'],
      \ [20, 21, 'lwonEscape'], [20, 23, 'lwonInvalidEscape'], [20, 25, 'lwonQuote'],
      \ [20, 26, 'lwonDelimiter'],
      \ [36, 1, 'lwonKey'], [36, 8, 'lwonQuote'], [36, 9, 'lwonString'],
      \ [37, 3, 'lwonComment'],
      \ [39, 3, 'lwonString'], [39, 15, 'lwonEscape'], [39, 22, 'lwonEscape'],
      \ [39, 31, 'lwonQuote'],
      \ [45, 1, 'lwonKey'], [45, 6, 'lwonShortString'], [45, 26, 'lwonShortString'],
      \ [49, 3, 'lwonQuote'], [49, 4, 'lwonString'], [49, 9, 'lwonSeparator'],
      \ [49, 11, 'lwonShortString'],
      \ [53, 3, 'lwonShortString'],
      \ [55, 1, 'lwonDelimiter'],
      \ [58, 1, 'lwonKey'], [58, 8, 'lwonQuote'], [58, 9, 'lwonQuote'],
      \ [61, 1, 'lwonKey'],
      \ [64, 1, 'lwonKey'], [64, 7, 'lwonKey'], [64, 9, 'lwonDelimiter'],
      \ [64, 11, 'lwonShortString'],
      \ [68, 1, 'lwonKey'], [68, 9, 'lwonDelimiter'], [68, 11, 'lwonKey'],
      \ [72, 1, 'lwonKey'], [72, 8, 'lwonKey'], [72, 10, 'lwonQuote'],
      \ ]

let s:failures = []
for [s:line, s:col, s:expected] in s:checks
  let s:actual = synIDattr(synID(s:line, s:col, 1), 'name')
  if s:actual !=# s:expected
    call add(s:failures, printf('line %d, col %d (%s): expected %s, got %s',
          \ s:line, s:col, getline(s:line)[s:col - 1], s:expected, s:actual))
  endif
endfor

if &commentstring !=# '# %s'
  call add(s:failures, 'commentstring is ' . &commentstring)
endif

call writefile(s:failures + [printf('%d of %d checks passed',
      \ len(s:checks) - len(s:failures), len(s:checks))], '/dev/stderr')
if empty(s:failures)
  qall!
endif
cquit
