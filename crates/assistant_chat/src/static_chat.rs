use std::sync::Arc;

use crate::ChatRole;
use indoc::indoc;
use nanoid::nanoid;

pub fn static_chat() -> Vec<(Arc<str>, ChatRole, Arc<str>)> {
    let message_1 = indoc! {r#"I want to write a python script to output all unique @{name} instancees across all highlight.scm files in my project. For example, @variable, @function, @class, etc.

    The script should output all unique instances of @{name} in the highlight.scm files, and addionally add a count to each.

    For example:

    ```
    @variable (100)
    @function (50)
    ```

    The list should be sorted in alphabetical order."#};

    let message_2 = indoc! {r#"Given your requirements, here's a Python script that will search for all unique instances of `@{name}` in `highlight.scm` files both in the core languages path and the extensions path. It will also count the occurrences of each instance and organize the output as specified:

    ```python
    def parse_arguments():
        parser = argparse.ArgumentParser(description='Analyze highlight.scm files for unique instances and their languages.')
        parser.add_argument('-v', '--verbose', action='store_true', help='Include a list of languages for each tag.')
        return parser.parse_args()

    def find_highlight_files(root_dir):
        for path in Path(root_dir).rglob('highlights.scm'):
            yield path

    def count_instances(files):
        instances: defaultdict[list[Any], dict[str, Any]] = defaultdict(lambda: {'count': 0, 'languages': set()})
        for file_path in files:
            language = file_path.parent.name
            with open(file_path, "r") as file:
                text = file.read()
                matches = pattern.findall(text)
                for match in matches:
                    instances[match]['count'] += 1
                    instances[match]['languages'].add(language)
        return instances
    ```

    To execute this script successfully, it must be placed within a directory as specified in your instructions (script/analyze_highlights.py), and the project's directory structure should match what you've described for both core languages and extensions."#};

    let message_3 = indoc! {r#"Ok, great. Let's add a -v or --verbose flag that will add a list of each language that uses that tag. The current output looks like this:

    ```
    @attribute (6)
    @boolean (5)
    @charset (1)
    @comment (19)
    ```

    In verbose mode, it should look like this:

    ```
    @attribute (6) - [java, python, ...]
    @boolean (5) - [java, python, ...]
    @charset (1) - [java]
    @comment (19) - [java, python, rust, ...]
    ```

    ... being the list of languages that use that tag."#};

    let message_4 = indoc! {r#"To modify your Python script to add the `-v` or `--verbose` flag for listing each language that uses a certain tag, you need to update the script to process command-line arguments, track which languages use which tags, and adjust the output based on the verbose flag. The argparse library in Python provides a convenient way to handle command-line arguments.

    ### Updated Python Script with Verbose Mode

    Below is the updated script with these enhancements:
    ```python
    from collections import defaultdict
    from pathlib import Path
    from typing import Any
    import argparse
    import re

    pattern = re.compile(r'@(?!_)[a-zA-Z_.]+')

    def parse_arguments():
        parser = argparse.ArgumentParser(description='Analyze highlight.scm files for unique instances and their languages.')
        parser.add_argument('-v', '--verbose', action='store_true', help='Include a list of languages for each tag.')
        return parser.parse_args()

    def find_highlight_files(root_dir):
        for path in Path(root_dir).rglob('highlights.scm'):
            yield path

    def count_instances(files):
        instances: defaultdict[list[Any], dict[str, Any]] = defaultdict(lambda: {'count': 0, 'languages': set()})
        for file_path in files:
            language = file_path.parent.name
            with open(file_path, "r") as file:
                text = file.read()
                matches = pattern.findall(text)
                for match in matches:
                    instances[match]['count'] += 1
                    instances[match]['languages'].add(language)
        return instances

    def print_instances(instances, verbose=False):
        for item, details in sorted(instances.items(), key=lambda x: x[0]):
            languages = ', '.join(sorted(details['languages']))
            if verbose:
                print(f"{item} ({details['count']}) - [{languages}]")
            else:
                print(f"{item} ({details['count']})")

    def main():
        args = parse_arguments()

        base_dir = Path(__file__).parent.parent
        core_path = base_dir / 'crates/languages/src'
        extension_path = base_dir / 'extensions/'

        core_instances = count_instances(find_highlight_files(core_path))
        extension_instances = count_instances(find_highlight_files(extension_path))

        unique_extension_instances = {k: v for k, v in extension_instances.items() if k not in core_instances}

        print('Shared:\n')
        print_instances(core_instances, args.verbose)

        if unique_extension_instances:
            print('\nExtension-only:\n')
            print_instances(unique_extension_instances, args.verbose)

    if __name__ == '__main__':
        main()
    ```

    This revised script now supports a `-v` or `--verbose` flag. When this flag is provided, the script will also print a list of languages associated with each tag in the format you specified. The language is assumed to be three levels up from the highlight.scm file, based on your project's directory structure. Adjustments may be necessary if this assumption is not correct for your project structure.

    I also added some documentation at the top of the script to help understand how it works.

    ```python
    """
    This script analyzes all the highlight.scm files in our embedded languages and extensions.
    It counts the number of unique instances of @{name} and the languages in which they are used.

    This is useful to help avoid accidentally introducing new tags when appropriate ones already exist when adding new languages.

    Flags:
    -v, --verbose: Include a detailed list of languages for each tag found in the highlight.scm files.
    """
    ```

    Let me know if there is anything else I can help with!"#};

    vec![
        (nanoid!().into(), ChatRole::User, message_1.into()),
        (nanoid!().into(), ChatRole::Assistant, message_2.into()),
        (nanoid!().into(), ChatRole::User, message_3.into()),
        (nanoid!().into(), ChatRole::Assistant, message_4.into()),
    ]
}
