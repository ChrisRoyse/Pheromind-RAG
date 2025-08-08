# Malformed Markdown Test - Parser Breaker 💥

This document contains intentionally malformed markdown designed to break parsers, renderers, and processors.

## Unclosed Tags and Mismatched Formatting

**This is bold but never closed

*This is italic but never closed

~~This is strikethrough but never closed

`This is inline code but never closed

```javascript
This is a code block that's never closed
function test() {
    return "unclosed";

## Nested Formatting Chaos

***This is**bold and italic*but**nested wrong***

**Bold with *italic inside**and more italic*

`code with **bold inside` and more bold**

~~strikethrough with `code inside~~ and more code`

## Broken Links and References

[This link has no URL]
[This link](has space in URL)
[This link](https://broken url with spaces.com)
![Broken image]
![Image with no URL]()
![](image with no alt text)

Reference links that don't exist: [undefined reference][nonexistent]
[Another broken][also nonexistent]

## Malformed Lists

- List item one
  - Nested item with wrong indentation
- List item two
  * Mixed list markers
  + More mixed markers
    - Inconsistent nesting
      * Even more nesting
        - Too much nesting
          * Way too much
            - This is ridiculous
              * Getting extreme
                - Maximum nesting

1. Ordered list
   1. Nested ordered
   3. Wrong number
   1. Back to one
2. Skip to two
5. Jump to five
a. Letter instead of number
1. Back to numbers

## Broken Tables

| Column 1 | Column 2 | Column 3
|----------|----------|
| Missing closing pipe
| Too many | pipes | in | this | row |
Completely broken table row
| Another | broken | table | row
|---------|---------|

| Header | Another |
|--------|
| Missing separator cells |
| More | cells | than | headers |

## Malformed HTML in Markdown

<div>
Unclosed div tag

<img src="broken.jpg" alt="unclosed img tag>
<p>Paragraph with missing closing tag
<strong>Bold with missing closing</strong>
<em>Italic with unclosed tag
<code>Code tag never closed

<script>
// Potentially dangerous script in markdown
alert('This should not execute');
</script>

<style>
/* Potentially dangerous CSS */
body { display: none; }
</style>

## Unicode Markdown Chaos

### Lists with Unicode bullets

• Unicode bullet point
‣ Another Unicode bullet  
◦ White bullet
▪ Black square bullet
▫ White square bullet
◆ Black diamond
◇ White diamond

### Headers with Unicode

# 中文标题 Header
## العربية عنوان Header
### русский заголовок Header
#### 日本語タイトル Header
##### 한국어 제목 Header

## Broken Code Blocks

```
Code block with no language specified
And multiple lines
```

```python
def broken_function():
    return "This code block has wrong indentation"
```

    Indented code block
    That continues
    ```
    With a nested code block inside
    ```
    Back to indented

## Malformed Blockquotes

> This is a blockquote
But this line is not properly quoted
> Back to quoted text
>> Double quoted
> But this is not double quoted
>>> Triple quoted
> Back to single
Not quoted at all

## Broken Emphasis and Strong

**Bold text with *italic inside but mismatched tags**
*Italic text with **bold inside but wrong closing*
***Triple emphasis that's broken**
____Quadruple underscore that's invalid____

## Invalid Characters in Markdown

Control characters in text: 
Null bytes in middle of text: Text with null  in middle
Tab characters that break formatting:	Tab	in	middle
Vertical tabs: Text with vertical tab
Form feed: Text with form feed

## Malformed Footnotes

This has a footnote[^1] that's defined.
This has a broken footnote[^broken] that's not defined.
This has a footnote with symbols[^symbol!@#$%] which is invalid.

Multiple footnotes pointing to same reference[^1][^1][^1].

[^1]: This is the footnote
[^2]: This footnote is never referenced
[^]: Footnote with no identifier

## Broken Mathematics (if supported)

$This is LaTeX but never closed
$$Double dollar math block never closed

$\frac{broken}{math$ with unmatched braces
$$
\begin{matrix}
Unmatched LaTeX environment
$$

## Malformed Definition Lists

Term 1
: Definition 1

Term 2
Definition without colon

: Definition without term

Term 3
: Definition 1
: Definition 2
: Too many definitions

## Nested Blockquotes Gone Wrong

> Level 1 quote
>> Level 2 quote
> Back to level 1 but formatting might be broken
>>> Level 3 from level 1?
> > Spaces in quote markers
>>>>> Way too many levels

## Invalid Escape Sequences

\Invalid escape at start
Text with \invalid escape in middle
\\\Multiple invalid escapes\\\
\*This should be literal asterisk
\[This should be literal bracket
\\But this is valid double backslash

## Broken Line Breaks

Line with trailing spaces but no break  
Proper line break

Line with trailing spaces but wrong break   \
Mixed break types

Hard line break\\
Soft line break\
Mixed breaks\\  

## Malformed Horizontal Rules

Not enough dashes: --
Wrong characters: ___***___
Mixed: ---***---
Spaces in wrong places: - - - - -
Too short: -

## Comments and Processing Instructions

<!-- This is a comment
<!-- Unclosed comment

<!-- Nested <!-- comment --> inside -->

<?xml version="1.0"?>
<xml>XML in markdown</xml>

<!DOCTYPE html>
<html>HTML in markdown</html>

## Charset and Encoding Issues

Filename with weird chars: файл名.md
Mixed encoding: caf├® r├®sum├® 
Broken UTF-8: ├┤─┘├┤─┘
Overlong UTF-8: ├┤───┘├┤───┘

## Binary Data in Markdown

This line contains binary: ������������
Mixed text and binary: Hello ����� World
Null bytes: Text with embedded  nulls

## Extremely Long Lines

This is an extremely long line that goes on and on and on and repeats the same content over and over to test buffer overflow and memory allocation limits in markdown processors that might not handle extremely long lines properly, causing them to crash or consume excessive memory when attempting to parse this ridiculously long line that seems to never end and continues indefinitely with the same repetitive content designed to stress test parsing systems and break markdown processing algorithms that assume reasonable line lengths but instead encounter this monstrosity of a line that just keeps going and going like the Energizer bunny but for text processing nightmare scenarios where normal assumptions about text structure completely break down and parsers have to deal with pathological input that no reasonable person would ever create except for the specific purpose of breaking things which is exactly what this line is designed to do by being unreasonably long and repetitive and annoying and designed to cause problems for any system that tries to process it in a normal way because normal markdown processing assumes that lines have reasonable lengths but this line violates that assumption completely and utterly and without any regard for the poor parser that has to deal with it and this continues for way too long just to make sure that any buffer overflow or memory allocation issues are triggered by the sheer length and repetitive nature of this pathological line that should break any markdown processor that doesn't properly handle extremely long lines.

## Deeply Nested Structures

- Level 1
  - Level 2
    - Level 3
      - Level 4
        - Level 5
          - Level 6
            - Level 7
              - Level 8
                - Level 9
                  - Level 10
                    - Level 11
                      - Level 12
                        - Level 13
                          - Level 14
                            - Level 15
                              - Level 16
                                - Level 17
                                  - Level 18
                                    - Level 19
                                      - Level 20
                                        - Beyond reasonable nesting

## Circular References

[This link points to itself][self]
[self]: #circular-references

[Loop 1][loop2]
[Loop 2][loop1]
[loop1]: #broken
[loop2]: #also-broken

## Invalid Table Structures

| Header |
|--------|--------|
| Too many separators |

|No header row|
|-------------|
| But has data |

| Misaligned | Headers    |
|------------|------------|-----------|
| Too many   | separators | in row    |

## Mixed Indentation Chaos

    Code block with 4 spaces
	Code block with tab
        Code block with 8 spaces
	    Mixed tab and spaces
 Code block with 1 space (invalid)
  Code block with 2 spaces (invalid)
   Code block with 3 spaces (invalid)

## Final Chaos Section

This final section combines multiple malformed elements:

**Bold with [broken link](missing url) and `unclosed code

> Blockquote with ***mismatched emphasis**
> 
> And a broken table:
| Column | Another
|--------
| Missing | cells

1. Ordered list with **bold
2. *italic never closed
   - Nested with ~~strikethrough
   - `Code in list never closed

```javascript
function chaos() {
    // Code block never closed
    return "broken";
```

<!-- Comment never closed

This is the end of the malformed markdown file. If your parser survived this, congratulations! 🎉

*But this italic is never closed either...