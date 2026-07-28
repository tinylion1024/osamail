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
    if (message.indexOf("OSAMAIL_ACCOUNT_MISMATCH") !== -1) {
        return failure(
            "INVALID_REQUEST",
            "Source and destination accounts must match."
        );
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

function findMessage(mailbox, locator) {
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

function samePath(left, right) {
    if (left.length !== right.length) {
        return false;
    }
    for (var index = 0; index < left.length; index += 1) {
        if (String(left[index]) !== String(right[index])) {
            return false;
        }
    }
    return true;
}

function run(argv) {
    try {
        if (argv.length !== 1) {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }
        var request = readRequest(argv[0]);
        if (
            request.operation !== "move_message" ||
            !request.locator ||
            request.locator.version !== 1 ||
            !request.locator.account ||
            !Array.isArray(request.locator.mailbox_path) ||
            request.locator.mailbox_path.length === 0 ||
            !request.destination ||
            request.destination.kind !== "mailbox" ||
            request.destination.version !== 1 ||
            !request.destination.account ||
            !Array.isArray(request.destination.mailbox_path) ||
            request.destination.mailbox_path.length === 0 ||
            typeof request.dry_run !== "boolean"
        ) {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }
        if (request.locator.account !== request.destination.account) {
            throw new Error("OSAMAIL_ACCOUNT_MISMATCH");
        }

        var Mail = Application("com.apple.mail");
        var account = findAccount(Mail, String(request.locator.account));
        var source = findMailbox(account, request.locator.mailbox_path);
        var destination = findMailbox(
            account,
            request.destination.mailbox_path
        );
        var message = findMessage(source, request.locator);
        var alreadyThere = samePath(
            request.locator.mailbox_path,
            request.destination.mailbox_path
        );

        if (!request.dry_run && !alreadyThere) {
            Mail.move(message, { to: destination });
        }

        return response({ already_there: alreadyThere });
    } catch (error) {
        return classify(error);
    }
}
