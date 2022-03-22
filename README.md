# subreddit-posts
This application will post at the chosen subreddits, can provide information about best time for each one
FOR NOW it posts only links;

## How
 1. You will need to create proper `.env` and `.posts` files
 2. Build up
 3. Run

### Env file
 You can check `.env.example`

### Posts file
You can check `.posts.example` 
and `.subreddits.example` for retrieve fleir ids

### Build app
 Install rust or docker

### Run
 cargo run, docker run


you will need to use 
`http://127.0.0.1:9090/reddit/login`
`http://127.0.0.1:9090/reddit/flairs`
`http://127.0.0.1:9090/reddit/post`
