from spin_sdk.http import Handler, Request, Response


class HttpHandler(Handler):
    async def handle_request(self, request: Request) -> Response:
        return Response(
            200,
            {"content-type": "text/plain"},
            bytes("Hello, this is Spin running on Akamai Functions!", "utf-8"),
        )
