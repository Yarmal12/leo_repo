// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

use crate::create_messages;

use std::{
    error::Error as ErrorArg,
    fmt::{Debug, Display},
};

create_messages!(
    /// CompilerError enum that represents all the errors for the `leo-compiler` crate.
    CompilerError,
    code_mask: 6000i32,
    code_prefix: "CMP",

    /// For when the compiler can't read a file from the provided path.
    @backtraced
    file_read_error {
        args: (path: impl Debug, error: impl ErrorArg),
        msg: format!("Cannot read from the provided file path '{:?}': {}", path, error),
        help: None,
    }

    /// For when a user tries to assign to a circuit static member.
    @formatted
    illegal_static_member_assignment {
        args: (member: impl Display),
        msg: format!("Tried to assign to static member `{member}`"),
        help: None,
    }

    @formatted
    import_not_found {
        args: (file_path: impl Display),
        msg: format!("Attempted to import a file that does not exist `{file_path}`."),
        help: None,
    }

    @formatted
    cannot_open_cwd {
        args: (err: impl ErrorArg),
        msg: format!("Failed to open current working directory. Error: {err}"),
        help: None,
    }
);