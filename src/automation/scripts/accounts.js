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

function strings(values) {
    var output = [];
    for (var index = 0; index < values.length; index += 1) {
        output.push(String(values[index]));
    }
    return output;
}

function classify(error) {
    var message = String(error);
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

function run(argv) {
    try {
        if (argv.length !== 1) {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }
        var request = readRequest(argv[0]);
        if (request.operation !== "accounts") {
            throw new Error("OSAMAIL_INVALID_REQUEST");
        }

        var Mail = Application("com.apple.mail");
        var source = Mail.accounts();
        var accounts = [];

        for (var index = 0; index < source.length; index += 1) {
            accounts.push({
                name: String(source[index].name()),
                email_addresses: strings(source[index].emailAddresses()),
                enabled: Boolean(source[index].enabled()),
            });
        }

        accounts.sort(function (left, right) {
            return left.name.localeCompare(right.name);
        });
        return response({ accounts: accounts });
    } catch (error) {
        return classify(error);
    }
}
