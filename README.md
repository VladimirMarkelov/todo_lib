# TODO Library

![](https://travis-ci.com/VladimirMarkelov/todo_lib.svg?branch=master)
[![](https://img.shields.io/crates/v/todo_lib.svg)](https://crates.io/crates/todo_lib)

A collection of utilities to work with files in [todo.txt format](http://todotxt.org/)

## Working with files

### Load todo list

` load(filename: &Path) -> Result<TaskVec, terr::TodoError>`

Loads a list of todos from a given file path. Returns the list of loaded todos or an error. If the file does not exist the function returns empty list.

### Save todo list

`save(tasks: &TaskSlice, filename: &Path) -> Result<(), terr::TodoError>`

Saves the todo list to a given file path. It may return an error if creating file or writing to it fails. The list is saved in two steps:

* a list is saved to a temporary file in the same directory with `filename`
* on success, it deleted the old file and renames temporary file to `filename`

### Archive completed todos

`archive(tasks: &TaskSlice, filename: &Path) -> Result<(), terr::TodoError>`

It works similar to `save` but appends task list to a given `filename`. It does not create a temporary file.

## Filtering

`filter(tasks: &todo::TaskSlice, c: &Conf) -> todo::IDVec`

The function gets the list of all todos and filtering rules and returns the list of todo IDs(todo's ID is the order number of the todo in the original list) that matches the rules. If a rule works with strings(e.g, projects or regex), the rule is case-insensitive. Available rules (if a rule is None the rule is skipped):

* `range` - selects one todo by its ID, or a few ones within ID range(inclusive), or a list of IDs;
* `all` - selects only all done, only incomplete, or both;
* `pri` - selects with any priority, without any priority, or with the same/higher/lower priority(inclusive);
* `regex` - when `use_regex` is true, it does regular expression pattern matching, otherwise it search for a substring. Note: it searches for the `regex` in subject, projects, and contexts;
* `projects` - selects all todos that have *any* of `projects`. This rule allows a caller to do very basic pattern matching: `*` added to the beginning or to the end of a project means to look for a project which name ends or starts respectively with the word, Adding `*` to both ends works like `regex` but checks only projects. `*` in the middle of the word does not have any special meaning - use `regex` in this case;
* `contexts` - selects all todos that have *any* of `contexts`. The rule can use `*` in the same way `projects` does;
* `tags` - selects all todos that have *any* of `tags`. The rule can use `*` in the same way `projects` does;
* `hashtags` - selects all todos that have *any* of `hashtags`. The rule can use `*` in the same way `projects` does;
* `due` - selects all todos with any due date, without due date, a todo with due date within range, or todos which are less than the number of days ahead;
* `rec` - selects all recurrent todos or all without recurrent flag.
* `thr` - selects all todos with any threshold date, without threshold date
* `tmr` - selects all active todos - that have their timers running
* `created` - selects all todos with any creation date, without creation date, a todo with creation date within range
* `finished` - selects all todos with any finish date, without finish date, a todo with finish date within range

Rules `contexts`, `projects`, `hashtags`, and `tags` support special values:

- `none` - filter todos that do not have any values (contexts=['none'] - todos without any context)
- `any` - filter todos that have at least one value (project=['any'] - todos that belong to any project)

## Sorting

`sort(ids: &mut todo::IDVec, todos: &todo::TaskSlice, c: &Conf)`

Because `sort` is the function that should be called after `filter`, it wants a list of selected todo IDs that must be sorted, the whole todo list (IDs in `ids` are the order numbers of a todo in `todos`) and sorting rules. The function changes `ids` in-place and the sorting is always stable - it keeps the order of todos in the `ids` list for todos that are equal to each other. There are only two sorting rules:

* `fields` - is a comma(or colon) separated list of fields in order of importance for sorting. If the vector is empty the list remains unchanged. Supported field names(and their abbreviations):
    - `pri` or `priority` - sort by priority (without priority are the last ones);
    - `due` - sort by due date (todos that do not have due date are at the bottom);
    - `completed` or `finished` - sort by completion date (incomplete ones are at the bottom);
    - `created` or `create` - sort by creation date;
    - `subject`, `subj` or `text` - sort by todo's subjects;
    - `done` - order: incomplete, recurrent, and done todos;
    - `project` or `proj` - sort by project names, if todos have more than one project they are compared in order of appearance and shorter list of projects goes first;
    - `context` or `ctx` - sort by contexts, if todos have more than one context they are compared in order of appearance and shorter list of contexts goes first;
    - `thr` - sort by threshold date (todos that do not have threshold date are at the bottom);
* `rev` - when it is `true` the sorted list is reversed before returning the result.

## Editing

### Add a new todo

`add(tasks: &mut TaskVec, c: &Conf) -> usize`

The function gets a list of existing todos and a `c` with non-None field `subject`, then it parses `subject` and adds the result todo to the list and return the ID of the new todo. If adding fails - e.g, `subject` is empty or parsing returns an error - the function returns `INVALID_ID`.

### Modify existing todos

Functions of this category gets a list of all todos, list of todo IDs that should be modified and optionally new values for properties. If list of IDs `ids` is None then the function modifies all todos. It returns the vector of boolean values with the length equal to the length of `ids` (or length of `tasks` if `ids` is None). If the result vector has `true` at some index it means that the todo from `ids` at the same index was modified.

#### Complete and undone todos

##### Mark a todo completed

`done(tasks: &mut TaskVec, ids: Option<&IDVec>, completion_config: todotxt::CompletionConfig) -> ChangedVec`

Makes all todos from `ids` list that are incomplete completed.

Special case: recurrent todos which contain due date and/or threshold date.
They are marked completed, and a new todos are created with their due and threshold dates moved to the next date in the future. But if a recurrent todo includes a tag `until` and its value less than the calculated due date for a new todo, a new todo is not created.

##### Remove completion mark from a todo

`undone(tasks: &mut TaskVec, ids: Option<&IDVec>, mode: todotxt::CompletionMode) -> ChangedVec`

Removes completion mark from all todos from `ids` list that are done.

Special case: recurrent todos. They are not changed.

#### Changing a specific property

`edit(tasks: &mut TaskVec, ids: Option<&IDVec>, c: &Conf) -> ChangedVec`

The function modifies all todos in `tasks` which IDs are in `ids` list. Note: modifying a subject changes only the first todo in `ids` list because it does not make sense to make all todos the same.

What can be modified:

* `subject` - set a new one
* `priority` - set, remove, increase or decrease priority
* `due date` - set or remove
 `thresold date` - set or remove
* `recurrence` - set or remove
* `projects` - add, remove or replace
* `contexts` - add, remove or replace
* `tags` - add, remove or replace
* `hashtags` - add, remove or replace

#### Time tracking support

To calculated time spent on a todo, two main functions are added:

##### Start and stop time tracking

`start(tasks: &mut TaskVec, ids: Option<&IDVec>) -> ChangedVec`

Makes todos active if they are incomplete and their is not yet running. All todos saves the timestamp when they were activated.

`stop(tasks: &mut TaskVec, ids: Option<&IDVec>) -> ChangedVec`

Stop timers for a given todos. Time taken by the todo is updated.

##### And two utility functions:

`is_timer_on(task: &todo_txt::task::Extended) -> bool`

Returns `true` if the given todo is active

`spent_time(task: &todo_txt::task::Extended) -> chrono::Duration`

Returns the total time spent on the given todo. For an inactive todo it returns the current value of todo's tag `spent`. For an active todo it returns sum of todo's tag `spent` and the time passed since the moment the todo was activated.
