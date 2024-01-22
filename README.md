# Simple HTTP server
\# Multithreaded http server \# 0 Dependencies
Install rust from here https://rustup.rs/
Once you have rust installed do cargo run in the top-level dir 
`cargo run`

This will make the server listen to 127.0.0.1:7878.
Use 127.0.0.1:7878 as a url in your browser

Feel free to modify hello.html to serve whatever
Also you can test the multithreaded ability with the 127.0.0.1:7878/sleep URL, this will wait 5 seconds before sending the response.

# Roadmap
- Host a webpage on this in a Docker instance on as Raspberri Pi at home
- Put my resume pdf somewhere there
- Fun?
- Use some APIs to fetch cool info?
- Use this to store my reading list :fire:
- Create daily logs, storing a week to month of data?
- Use types and regex to decompose HTTP requests
# License
MIT
