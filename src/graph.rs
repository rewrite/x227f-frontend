use std::collections::{HashMap, VecDeque};

use duckdb::Connection;

/// Page and button ids, as indexes into `Graph::pages` and `Graph::buttons`.
type Id = u32;

pub struct Graph {
    pages: Vec<Box<str>>,
    page_ids: HashMap<Box<str>, Id>,
    buttons: Vec<Box<str>>,
    /// `(linked page, button)` for each page.
    out: Vec<Vec<(Id, Id)>>,
}

/// One page on a path, with the button that leads to the next page.
pub struct Step<'a> {
    pub page: &'a str,
    /// The button on this page that links to the next one (None on the last page).
    pub button: Option<&'a str>,
}

impl Graph {
    pub fn load(db: &Connection) -> duckdb::Result<Self> {
        let mut graph = Graph {
            pages: Vec::new(),
            page_ids: HashMap::new(),
            buttons: Vec::new(),
            out: Vec::new(),
        };
        let mut button_ids: HashMap<String, Id> = HashMap::new();

        let mut stmt = db.prepare(
            "SELECT page, link, min(sha256) FROM pages \
             WHERE link IS NOT NULL AND page <> link GROUP BY page, link ORDER BY page, link",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let from = graph.intern_page(row.get_ref(0)?.as_str()?);
            let to = graph.intern_page(row.get_ref(1)?.as_str()?);
            let sha: String = row.get(2)?;
            let next = graph.buttons.len() as Id;
            let button = *button_ids.entry(sha).or_insert_with_key(|sha| {
                graph.buttons.push(sha.as_str().into());
                next
            });
            graph.out[from as usize].push((to, button));
        }
        Ok(graph)
    }

    fn intern_page(&mut self, page: &str) -> Id {
        if let Some(&id) = self.page_ids.get(page) {
            return id;
        }
        let id = self.pages.len() as Id;
        self.pages.push(page.into());
        self.page_ids.insert(page.into(), id);
        self.out.push(Vec::new());
        id
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn edge_count(&self) -> usize {
        self.out.iter().map(Vec::len).sum()
    }

    /// Finds a page the way people tend to type it: with or without the
    /// scheme, `www.` or a trailing slash.
    pub fn find_page(&self, input: &str) -> Option<&str> {
        let s = input.trim();
        let s = s.strip_prefix("https://").or_else(|| s.strip_prefix("http://")).unwrap_or(s);
        let s = s.trim_end_matches('/');
        let bare = s.strip_prefix("www.").unwrap_or(s);
        let candidates = [s.to_owned(), format!("{s}/"), bare.to_owned(), format!("www.{bare}")];
        candidates
            .iter()
            .find_map(|c| self.page_ids.get_key_value(c.as_str()))
            .map(|(k, _)| &**k)
    }

    /// Shortest path from `from` to `to` by breadth-first search over button
    /// links. Both pages must come from `find_page`.
    pub fn shortest_path(&self, from: &str, to: &str) -> Option<Vec<Step<'_>>> {
        let (from, to) = (self.page_ids[from], self.page_ids[to]);
        // parent[page] = (previous page, its button linking here); from is its own parent
        let mut parent: Vec<Option<(Id, Id)>> = vec![None; self.pages.len()];
        parent[from as usize] = Some((from, 0));
        let mut queue = VecDeque::from([from]);

        'search: while let Some(page) = queue.pop_front() {
            for &(next, button) in &self.out[page as usize] {
                if parent[next as usize].is_none() {
                    parent[next as usize] = Some((page, button));
                    if next == to {
                        break 'search;
                    }
                    queue.push_back(next);
                }
            }
        }
        parent[to as usize]?;

        let mut path = vec![Step { page: &self.pages[to as usize], button: None }];
        let mut page = to;
        while page != from {
            let (prev, button) = parent[page as usize].unwrap();
            path.push(Step { page: &self.pages[prev as usize], button: Some(&self.buttons[button as usize]) });
            page = prev;
        }
        path.reverse();
        Some(path)
    }
}
