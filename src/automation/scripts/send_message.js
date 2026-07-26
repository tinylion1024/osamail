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

function findAccount(Mail, name) {
    var accounts = Mail.accounts();
    for (var index = 0; index < accounts.length; index += 1) {
        if (
            String(accounts[index].name()) === name &&
            Boolean(accounts[index].enabled())
        ) {
            return accounts[index];
        }
    }
    throw new Error("OSAMAIL_ACCOUNT_NOT_FOUND");
}

function addRecipients(Mail, collection, values, recipientType) {
    for (var index = 0; index < values.length; index += 1) {
        var recipient;
        if (recipientType === "to") {
            recipient = Mail.ToRecipient({ address: String(values[index]) });
        } else if (recipientType === "cc") {
            recipient = Mail.CcRecipient({ address: String(values[index]) });
        } else {
            recipient = Mail.BccRecipient({ address: String(values[index]) });
        }
        collection.push(recipient);
    }
}

function run(argv) {
    try {
        if (argv.length !== 1) {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }
        var request = readRequest(argv[0]);
        if (
            request.operation !== "send_message" ||
            !Array.isArray(request.to) ||
            !Array.isArray(request.cc) ||
            !Array.isArray(request.bcc) ||
            request.to.length + request.cc.length + request.bcc.length === 0 ||
            typeof request.subject !== "string" ||
            typeof request.body !== "string"
        ) {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }

        var Mail = Application("com.apple.mail");
        var sender = null;
        if (request.account) {
            var account = findAccount(Mail, String(request.account));
            var addresses = account.emailAddresses();
            if (addresses.length === 0) {
                throw new Error("OSAMAIL_ACCOUNT_NOT_FOUND");
            }
            sender = String(addresses[0]);
        }

        var properties = {
            subject: request.subject,
            content: request.body,
            visible: false,
        };
        if (sender) {
            properties.sender = sender;
        }
        var draft = Mail.OutgoingMessage(properties);
        Mail.outgoingMessages.push(draft);

        addRecipients(Mail, draft.toRecipients, request.to, "to");
        addRecipients(Mail, draft.ccRecipients, request.cc, "cc");
        addRecipients(Mail, draft.bccRecipients, request.bcc, "bcc");

        var accepted = Boolean(draft.send());
        if (!accepted) {
            throw new Error("OSAMAIL_SEND_REJECTED");
        }
        return response({
            sent: true,
            dry_run: false,
            account: request.account || null,
            recipient_count:
                request.to.length + request.cc.length + request.bcc.length,
        });
    } catch (error) {
        return classify(error);
    }
}
