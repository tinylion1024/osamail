ObjC.import("Foundation");

function response(data) {
    return JSON.stringify({ ok: true, data: data });
}

function failure(code, message) {
    return JSON.stringify({ ok: false, error: { code: code, message: message } });
}

function readRequest(path) {
    var text = $.NSString.stringWithContentsOfFileEncodingError(
        path,
        $.NSUTF8StringEncoding,
        null
    );
    if (!text) {
        throw new Error("OSAMAIL_INVALID_REQUEST");
    }
    return JSON.parse(ObjC.unwrap(text));
}

function classify(error) {
    var message = String(error);
    if (message.indexOf("OSAMAIL_ACCOUNT_NOT_FOUND") !== -1) {
        return failure("ACCOUNT_NOT_FOUND", "Account not found.");
    }
    if (message.indexOf("OSAMAIL_MAILBOX_NOT_FOUND") !== -1) {
        return failure("MAILBOX_NOT_FOUND", "Mailbox not found.");
    }
    if (
        message.indexOf("-1743") !== -1 ||
        message.indexOf("Not authorized") !== -1 ||
        message.indexOf("not authorized") !== -1 ||
        message.indexOf("not permitted") !== -1
    ) {
        return failure(
            "AUTOMATION_PERMISSION_DENIED",
            "Apple Events automation permission was denied."
        );
    }
    if (message.indexOf("OSAMAIL_INVALID_REQUEST") !== -1) {
        return failure("INVALID_REQUEST", "The automation request is invalid.");
    }
    return failure("SCRIPT_FAILED", "Apple Mail automation failed.");
}

function findAccount(Mail, name) {
    var accounts = Mail.accounts();
    for (var index = 0; index < accounts.length; index += 1) {
        if (String(accounts[index].name()) === name) {
            return accounts[index];
        }
    }
    throw new Error("OSAMAIL_ACCOUNT_NOT_FOUND");
}

function collectNamedMailboxes(collection, name, output) {
    var boxes = collection();
    for (var index = 0; index < boxes.length; index += 1) {
        var box = boxes[index];
        if (String(box.name()) === name) {
            output.push(box);
        }
        collectNamedMailboxes(box.mailboxes, name, output);
    }
}

function selectedMailboxes(Mail, request) {
    if (!request.mailbox) {
        return [Mail.inbox()];
    }

    var matches = [];
    if (request.account) {
        var account = findAccount(Mail, String(request.account));
        collectNamedMailboxes(account.mailboxes, String(request.mailbox), matches);
    } else {
        var accounts = Mail.accounts();
        for (var index = 0; index < accounts.length; index += 1) {
            collectNamedMailboxes(
                accounts[index].mailboxes,
                String(request.mailbox),
                matches
            );
        }
    }

    if (matches.length === 0) {
        throw new Error("OSAMAIL_MAILBOX_NOT_FOUND");
    }
    return matches;
}

function mailboxPath(mailbox) {
    var path = [];
    var cursor = mailbox;

    for (var depth = 0; depth < 64 && cursor; depth += 1) {
        path.unshift(String(cursor.name()));
        try {
            var parent = cursor.container();
            if (!parent) {
                break;
            }
            cursor = parent;
        } catch (error) {
            break;
        }
    }
    return path;
}

function optionalString(value) {
    try {
        var resolved = value;
        if (typeof value === "function") {
            resolved = value();
        }
        if (resolved === null || resolved === undefined) {
            return null;
        }
        var text = String(resolved);
        return text.length === 0 ? null : text;
    } catch (error) {
        return null;
    }
}

function optionalDate(value) {
    try {
        var date = typeof value === "function" ? value() : value;
        if (!date) {
            return null;
        }
        return new Date(date).toISOString();
    } catch (error) {
        return null;
    }
}

function asArray(value) {
    if (Array.isArray(value)) {
        return value;
    }
    if (value === null || value === undefined) {
        return [];
    }
    return [value];
}

function containsFold(value, query) {
    return String(value || "")
        .toLocaleLowerCase()
        .indexOf(String(query).toLocaleLowerCase()) !== -1;
}

function bodySearchSource(source, request) {
    if (!request.search_body || !request.query) {
        return source;
    }
    return source.whose({
        _or: [
            { subject: { _contains: String(request.query) } },
            { sender: { _contains: String(request.query) } },
            { content: { _contains: String(request.query) } },
        ],
    });
}

function messageRows(mailboxes, request) {
    var candidates = [];
    var count = 0;
    var needsUnread = request.mode === "unread" || Boolean(request.unread);
    var needsOutput = !request.count_only;

    for (var boxIndex = 0; boxIndex < mailboxes.length; boxIndex += 1) {
        var source = bodySearchSource(mailboxes[boxIndex].messages, request);
        var dates = needsOutput ? asArray(source.dateReceived()) : [];
        var readStatuses =
            needsOutput || needsUnread ? asArray(source.readStatus()) : [];
        var senders =
            needsOutput || request.from || request.query
                ? asArray(source.sender())
                : [];
        var subjects =
            needsOutput || request.subject || request.query
                ? asArray(source.subject())
                : [];
        var messageIds = needsOutput ? asArray(source.id()) : [];
        var internetMessageIds = needsOutput
            ? asArray(source.messageId())
            : [];
        var mailboxNames =
            needsOutput && !request.mailbox
                ? asArray(source.mailbox.name())
                : [];
        var accountNames = needsOutput || request.account
            ? asArray(source.mailbox.account.name())
            : [];
        var selectedMailboxPath = request.mailbox
            ? mailboxPath(mailboxes[boxIndex])
            : null;
        var sourceCount = Math.max(
            dates.length,
            readStatuses.length,
            senders.length,
            subjects.length,
            messageIds.length,
            internetMessageIds.length,
            mailboxNames.length,
            accountNames.length,
            Number(source.length)
        );

        for (var messageIndex = 0; messageIndex < sourceCount; messageIndex += 1) {
            var sender = optionalString(senders[messageIndex]) || "";
            var subject = optionalString(subjects[messageIndex]) || "";

            if (
                request.account &&
                String(accountNames[messageIndex]) !== String(request.account)
            ) {
                continue;
            }
            if (needsUnread && Boolean(readStatuses[messageIndex])) {
                continue;
            }
            if (request.from && !containsFold(sender, request.from)) {
                continue;
            }
            if (request.subject && !containsFold(subject, request.subject)) {
                continue;
            }
            if (
                request.query &&
                !request.search_body &&
                !containsFold(subject, request.query) &&
                !containsFold(sender, request.query)
            ) {
                continue;
            }

            count += 1;
            if (!needsOutput) {
                continue;
            }

            var receivedAt = optionalDate(dates[messageIndex]);
            candidates.push({
                _received_time: receivedAt ? Date.parse(receivedAt) : 0,
                locator: {
                    version: 1,
                    account: String(accountNames[messageIndex]),
                    mailbox_path: selectedMailboxPath || [
                        String(mailboxNames[messageIndex]),
                    ],
                    message_id: Number(messageIds[messageIndex]),
                    internet_message_id: optionalString(
                        internetMessageIds[messageIndex]
                    ),
                },
                sender: sender,
                subject: subject,
                received_at: receivedAt,
                unread: !Boolean(readStatuses[messageIndex]),
            });
        }
    }

    candidates.sort(function (left, right) {
        return right._received_time - left._received_time;
    });
    candidates = candidates.slice(0, Number(request.limit));

    var rows = [];
    for (var index = 0; index < candidates.length; index += 1) {
        var candidate = candidates[index];
        rows.push({
            locator: candidate.locator,
            sender: candidate.sender,
            subject: candidate.subject,
            received_at: candidate.received_at,
            unread: candidate.unread,
        });
    }
    return { rows: rows, count: count };
}

function run(argv) {
    try {
        if (argv.length !== 1) {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }
        var request = readRequest(argv[0]);
        if (request.operation !== "list_messages") {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }
        if (
            request.mode !== "recent" &&
            request.mode !== "unread" &&
            request.mode !== "search"
        ) {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }
        if (Number(request.limit) < 1 || Number(request.limit) > 200) {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }

        var Mail = Application("com.apple.mail");
        if (request.account) {
            findAccount(Mail, String(request.account));
        }
        var mailboxes = selectedMailboxes(Mail, request);

        if (
            request.count_only &&
            !request.account &&
            !request.mailbox &&
            request.mode === "unread" &&
            !request.query &&
            !request.from &&
            !request.subject
        ) {
            return response({
                messages: [],
                count: Number(Mail.inbox().unreadCount()),
            });
        }

        var result = messageRows(mailboxes, request);
        if (request.count_only) {
            return response({ messages: [], count: result.count });
        }
        return response({ messages: result.rows, count: result.count });
    } catch (error) {
        return classify(error);
    }
}
