# Outlaw Format

A formatter for the syntax of the [vim-outlaw](https://github.com/lifepillar/vim-outlaw) outliner.

## How to use

1. Place the binary somewhere in your `PATH` and rename it to `outlaw-format`
2. Put these lines in `~/.vim/after/ftplugin/outlaw.vim`:

        " Format file on save
        function! s:outlaw_format()
            if executable('outlaw-format') == 0
                return
            endif

            let l:view = winsaveview()
            keepjumps execute '%!outlaw-format'
            call winrestview(l:view)
        endfunction
        autocmd! BufWritePre <buffer> :call s:outlaw_format()

## Style choices

Because *vim-outlaw* uses indentation to indicate the hierarchy of a document (similar to, say, Python), the most important formatting rule has to do with how it determines the width of a line.

Normally you would have a fixed line width (`textwidth` in vim) for the entire document but this is problematic for deeper levels of a Outlaw document. 

Because deeper levels have a (increasingly large) part of the fixed line width used up by indentation, the text on the line itself becomes narrower for each extra level, like a funnel. A simplified example with a line width of 64:

    === Header 1
        
        Lorem ipsum dolor sit amet, consectetur adipiscing elit.

        === Header 2

            Lorem ipsum dolor sit amet, consectetur adipiscing
            elit.

            [...]

                        === Header 6

                            Lorem ipsum dolor sit amet,
                            consectetur adipiscing elit.

I think deeper levels of a well-structured document are just as important (or maybe even more so) as the upper levels so this formatter makes text on all levels equally wide. In the simplified example above the text at header 6 would not be wrapped, just like it wasn't at header 1.

I would recommend setting `setlocal textwidth=0` for Outlaw documents in vim and let this formatter do the line wrapping instead, otherwise you have to battle vim's wrapping that doesn't take the amount of indentation into account.

Another notable formatting rule is that lists (`*`) are allowed to have a hierarchy too. I like lists for quickly taking notes and using indentation in them to add some context. Once they grow big enough I split them up into separate headers but for quick note taking I find it ideal.

An example of a list that is accepted by the formatter:

    * Lorem ipsum dolor sit amet, consectetur adipiscing elit
    * Mauris nibh erat, pulvinar ac porta sed, tempor sit amet augue
        * Duis vel quam massa!
            * Nulla nec ex neque?
    * Aliquam id ullamcorper nisl, nec sollicitudin sem

Other formatting rules are based on my personal preferences and have to do with whitespacing.

The "fenced filetypes" feature (syntax highlighted blocks of text) of *vim-outlaw* is supported and the formatter doesn't try to reformat their contents.

TODO lists (list items that start with `[ ]` or `[x]`) are also supported.
