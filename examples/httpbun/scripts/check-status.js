// A response handler kept in its own file. Saving an edited handler in
// Broquest writes it back here.
client.test("status is 2xx", function () {
  client.assert(response.status >= 200 && response.status < 300, "got " + response.status);
});
client.log("checked", response.status, response.contentType.mimeType);
