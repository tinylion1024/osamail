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

function selectedAccounts(Mail, requestedName) {
    var accounts = Mail.accounts();
    if (requestedName === null) {
        return accounts;
    }
    for (var index = 0; index < accounts.length; index += 1) {
        if (String(accounts[index].name()) === requestedName) {
            return [accounts[index]];
        }
    }
    throw new Error("OSAMAIL_ACCOUNT_NOT_FOUND");
}

function appendMailboxes(output, accountName, collection, parentPath) {
    var mailboxes = collection();
    for (var index = 0; index < mailboxes.length; index += 1) {
        var mailbox = mailboxes[index];
        var path = parentPath.concat([String(mailbox.name())]);
        output.push({ account: accountName, path: path });
        appendMailboxes(output, accountName, mailbox.mailboxes, path);
    }
}

function run(argv) {
    try {
        if (argv.length !== 1) {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }
        var request = readRequest(argv[0]);
        if (
            request.operation !== "list_mailboxes" ||
            !Object.prototype.hasOwnProperty.call(request, "account") ||
            (request.account !== null && typeof request.account !== "string")
        ) {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }

        var Mail = Application("com.apple.mail");
        var accountName =
            request.account === null ? null : String(request.account);
        var accounts = selectedAccounts(Mail, accountName);
        var output = [];
        for (var index = 0; index < accounts.length; index += 1) {
            var account = accounts[index];
            var name = String(account.name());
            appendMailboxes(output, name, account.mailboxes, []);
        }
        return response({ mailboxes: output });
    } catch (error) {
        return classify(error);
    }
}
