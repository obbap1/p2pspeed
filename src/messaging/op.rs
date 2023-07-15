enum Op {
    Request, // this is when a new peer is asking for historical information
    Response, // This is a peer responding to another peer with historical information
    Record // this is a peer doing speed tests at interval X and keeping them for record purposes.
}

