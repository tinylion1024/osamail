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
    if (message.indexOf("OSAMAIL_MESSAGE_NOT_FOUND") !== -1) {
        return failure("MESSAGE_NOT_FOUND", "Message not found.");
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

function findMailbox(account, path) {
    var collection = account.mailboxes;
    var mailbox = null;
    for (var depth = 0; depth < path.length; depth += 1) {
        var boxes = collection();
        mailbox = null;
        for (var index = 0; index < boxes.length; index += 1) {
            if (String(boxes[index].name()) === String(path[depth])) {
                mailbox = boxes[index];
                break;
            }
        }
        if (!mailbox) {
            throw new Error("OSAMAIL_MAILBOX_NOT_FOUND");
        }
        collection = mailbox.mailboxes;
    }
    return mailbox;
}

function findMessage(Mail, locator) {
    var account = findAccount(Mail, String(locator.account));
    var mailbox = findMailbox(account, locator.mailbox_path);
    var messages = mailbox.messages.whose({ id: Number(locator.message_id) })();
    if (messages.length === 0) {
        throw new Error("OSAMAIL_MESSAGE_NOT_FOUND");
    }
    var message = messages[0];
    if (
        locator.internet_message_id &&
        String(message.messageId()) !== String(locator.internet_message_id)
    ) {
        throw new Error("OSAMAIL_MESSAGE_NOT_FOUND");
    }
    return message;
}

function optionalString(value) {
    var resolved = value();
    if (resolved === null || resolved === undefined) {
        return null;
    }
    var text = String(resolved);
    return text.length === 0 ? null : text;
}

function optionalDate(value) {
    var date = value();
    return date ? new Date(date).toISOString() : null;
}

function recipients(collection) {
    var source = collection();
    var values = [];
    for (var index = 0; index < source.length; index += 1) {
        var name = optionalString(source[index].name);
        var address = optionalString(source[index].address) || "";
        if (name && address) {
            values.push(name + " <" + address + ">");
        } else {
            values.push(address || name || "");
        }
    }
    return values;
}

function run(argv) {
    try {
        if (argv.length !== 1) {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }
        var request = readRequest(argv[0]);
        if (
            request.operation !== "show_message" ||
            !request.locator ||
            request.locator.version !== 1 ||
            !request.locator.account ||
            !Array.isArray(request.locator.mailbox_path) ||
            request.locator.mailbox_path.length === 0
        ) {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }

        var Mail = Application("com.apple.mail");
        var message = findMessage(Mail, request.locator);
        var headers = null;
        if (request.include_headers) {
            headers = optionalString(message.allHeaders);
        }

        return response({
            locator: request.locator,
            sender: optionalString(message.sender) || "",
            to: recipients(message.toRecipients),
            cc: recipients(message.ccRecipients),
            bcc: recipients(message.bccRecipients),
            subject: optionalString(message.subject) || "",
            received_at: optionalDate(message.dateReceived),
            unread: !Boolean(message.readStatus()),
            body: optionalString(message.content) || "",
            headers: headers,
            message_id: optionalString(message.messageId),
        });
    } catch (error) {
        return classify(error);
    }
}
