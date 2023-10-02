use std::{env, fs, iter::Peekable, str::CharIndices, fmt::Debug};

#[derive(PartialEq)]
enum BlockContent<'a> {
    Text(&'a str),
    InnerBlock(Block<'a>)
}

impl Debug for BlockContent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockContent::Text(s) => f.write_str(s),
            BlockContent::InnerBlock(b) => b.fmt(f)
        }
    }
}

#[derive(PartialEq)]
struct Block<'a> {
    content: Vec<BlockContent<'a>>
}

impl Debug for Block<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.content.fmt(f)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_contents = fs::read_to_string(&args[1]);

    if file_contents.is_ok() {
        let txt = file_contents.unwrap();
        // let separated_file_contents = separate_by_delimiters(&txt, vec![' ', ',', '\n']);
        let separated_file_blocks = separate_by_blocks(&txt);

        println!("{:?}", separated_file_blocks)
    } else {
        println!("{:?}", file_contents)
    }
}

fn separate_by_blocks(s: &str) -> Result<Block, String> {
    let delimiters: Vec<(char, char)> = vec![('{', '}')];
    let mut content = Vec::<BlockContent>::new();

    let mut index = 0;

    let mut char_iter = s.char_indices().peekable();
    loop {
        let next = &char_iter.next();
        if next.is_none() {
            if index < s.len() {
                content.push(BlockContent::Text(&s[index..]));
            }
            return Ok(Block {content});
        }
        let (i, c) = next.unwrap();
        // Find another block start delimiter -> call encapsulate block on it and skip the iterator
        // to its end
        for delim in &delimiters {
            // c/i is the start of a new block
            if delim.0 == c {
                if i != index {
                    content.push(BlockContent::Text(&s[index..i]));
                }
                let block = encapsulate_block(s, &mut char_iter, i + 1, delim.1, &delimiters)?;
                let peek = &char_iter.peek();
                if peek.is_some() {
                    index = peek.unwrap().0;
                } else {
                    index = s.len()
                }
                content.push(BlockContent::InnerBlock(block));
            }
        }
    }
}

fn encapsulate_block<'a>(s: &'a str, char_iter: &mut Peekable<CharIndices>, start_index: usize, delimiter: char, delimiters: &Vec<(char, char)>) -> Result<Block<'a>, String> {
    let mut content = Vec::<BlockContent>::new();

    let mut index = start_index;

    println!("{}", s);
    loop {
        let next = &char_iter.next();
        if next.is_none() {
            return Err(String::from("UNABLE TO FINISH BLOCK"));
        }
        let (i, c) = next.unwrap();
        // Find delimiter -> Push end of block to content and return 
        if c == delimiter {
            content.push(BlockContent::Text(&s[index..i]));
            return Ok(Block {content});
        }
        // Find another block start delimiter -> call encapsulate block on it and skip the iterator
        // to its end
        for delim in delimiters {
            // c/i is the start of a new block
            if delim.0 == c {
                if i != index {
                    content.push(BlockContent::Text(&s[index..i]));
                }
                index = i;
                let block = encapsulate_block(s, char_iter, i + 1, delim.1, delimiters)?;
                let peek = &char_iter.peek();
                if peek.is_some() {
                    index = peek.unwrap().0;
                }
                content.push(BlockContent::InnerBlock(block));
            }
        }
    }
}


fn separate_by_delimiters(s: &str, delimiters: Vec<char>) -> Vec<&str> {
    let mut strings = Vec::<&str>::new();

    let mut index = 0;

    s.char_indices().for_each(|(i, c)| {
        if delimiters.contains(&c) {
            if i != index {
                strings.push(&s[index..i])
            }
            index = i + 1;
        }
    });

    strings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_inner_blocks() -> Result<(), String> {
        let res = separate_by_blocks("\"use strict\";")?;

        assert_eq!(res.content.len(), 1);
        let content = &res.content[0];
        let text = match content {
            BlockContent::InnerBlock(_) => Err(String::from("There should be no inner block!")),
            BlockContent::Text(t) => Ok(t)
        }?;

        assert_eq!(*text, "\"use strict\";");

        Ok(())
    }

    #[test]
    fn one_inner_block() -> Result<(), String> {
        let res = separate_by_blocks("\"use strict\";{dawha}")?;

        assert_eq!(res.content.len(), 2);
        let content = &res.content[0];
        let text = match content {
            BlockContent::InnerBlock(_) => Err(String::from("There should be no inner block!")),
            BlockContent::Text(t) => Ok(t)
        }?;

        assert_eq!(*text, "\"use strict\";");

        let content = &res.content[1];
        let block = match content {
            BlockContent::InnerBlock(b) => Ok(b),
            BlockContent::Text(_) => Err(String::from("There should be an inner block!"))
        }?;
        assert_eq!(block.content.len(), 1);

        let text = match block.content[0] {
            BlockContent::InnerBlock(_) => Err(String::from("There should be no inner block!")),
            BlockContent::Text(t) => Ok(t)
        }?;
        assert_eq!(text, "dawha");

        Ok(())
    }

    #[test]
    fn two_inner_blocks() -> Result<(), String> {
        let res = separate_by_blocks("\"use strict\";{dawha} export function dadwa() {\ndhadwajd;\n}")?;

        assert_eq!(res, Block {
            content: vec![
                BlockContent::Text("\"use strict\";"),
                BlockContent::InnerBlock(Block { content: vec![
                    BlockContent::Text("dawha")] }),
                BlockContent::Text(" export function dadwa() "),
                BlockContent::InnerBlock(Block { content: vec![BlockContent::Text("\ndhadwajd;\n")]})
            ]
        });

        Ok(())
    }

    #[test]
    fn nested_inner_blocks() -> Result<(), String> {
        let res = separate_by_blocks("\"use strict\";{dawha {something else reallyy cool } something else}")?;

        assert_eq!(res, Block {
            content: vec![
                BlockContent::Text("\"use strict\";"),
                BlockContent::InnerBlock(Block { content: vec![
                    BlockContent::Text("dawha "),
                    BlockContent::InnerBlock(Block { content: vec![ BlockContent::Text("something else reallyy cool ")]}),
                    BlockContent::Text(" something else"),
            ] }),
            ]
        });

        Ok(())
    }
}
