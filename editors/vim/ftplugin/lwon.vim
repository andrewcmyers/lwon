" Vim filetype plugin
" Language: LWON (Light-Weight Object Notation)

if exists('b:did_ftplugin')
  finish
endif
let b:did_ftplugin = 1

setlocal comments=:# commentstring=#\ %s
let b:undo_ftplugin = 'setlocal comments< commentstring<'
