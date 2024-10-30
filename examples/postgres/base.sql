CREATE DATABASE prod;

\c prod;

CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL
);

CREATE TABLE posts (
  id SERIAL PRIMARY KEY,
  title VARCHAR(255) NOT NULL,
  body TEXT NOT NULL,
  user_id INTEGER REFERENCES users(id)
);

CREATE TABLE comments (
  id SERIAL PRIMARY KEY,
  body TEXT NOT NULL,
  post_id INTEGER REFERENCES posts(id)
);

CREATE TABLE tags (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL
);

CREATE TABLE post_tags (
  post_id INTEGER REFERENCES posts(id),
  tag_id INTEGER REFERENCES tags(id),
  PRIMARY KEY (post_id, tag_id)
);

CREATE TABLE likes (
  user_id INTEGER REFERENCES users(id),
  post_id INTEGER REFERENCES posts(id),
  PRIMARY KEY (user_id, post_id)
);

CREATE TABLE followers (
  follower_id INTEGER REFERENCES users(id),
  followee_id INTEGER REFERENCES users(id),
  PRIMARY KEY (follower_id, followee_id)
);

CREATE TABLE notifications (
  id SERIAL PRIMARY KEY,
  user_id INTEGER REFERENCES users(id),
  message TEXT NOT NULL
);

CREATE TABLE messages (
  id SERIAL PRIMARY KEY,
  sender_id INTEGER REFERENCES users(id),
  receiver_id INTEGER REFERENCES users(id),
  body TEXT NOT NULL
);

CREATE TABLE sessions (
  id SERIAL PRIMARY KEY,
  user_id INTEGER REFERENCES users(id),
  token VARCHAR(255) NOT NULL
);

CREATE TABLE settings (
  id SERIAL PRIMARY KEY,
  user_id INTEGER REFERENCES users(id),
  theme VARCHAR(255) NOT NULL
);

CREATE TABLE reports (
  id SERIAL PRIMARY KEY,
  user_id INTEGER REFERENCES users(id),
  post_id INTEGER REFERENCES posts(id),
  reason TEXT NOT NULL
);

CREATE TABLE blocks (
  blocker_id INTEGER REFERENCES users(id),
  blockee_id INTEGER REFERENCES users(id),
  PRIMARY KEY (blocker_id, blockee_id)
);

CREATE TABLE mutes (
  muter_id INTEGER REFERENCES users(id),
  mutee_id INTEGER REFERENCES users(id),
  PRIMARY KEY (muter_id, mutee_id)
);

CREATE INDEX "idx_user_email" ON users (email);
CREATE INDEX "idx_user_name_email" ON users (name, email);

CREATE INDEX "idx_post_user_id" ON posts (user_id);
CREATE INDEX "idx_comment_post_id" ON comments (post_id);
CREATE INDEX "idx_post_tag_post_id" ON post_tags (post_id);
CREATE INDEX "idx_post_tag_tag_id" ON post_tags (tag_id);
CREATE INDEX "idx_like_user_id" ON likes (user_id);
CREATE INDEX "idx_like_post_id" ON likes (post_id);

CREATE TABLE key_values (
  key SERIAL PRIMARY KEY,
  value TEXT NOT NULL
);