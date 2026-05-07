ALTER TABLE messages
    DROP column media_url,
    DROP column file_id,
    DROP column media_group_id,
    DROP column payload,
    ADD column text TEXT;