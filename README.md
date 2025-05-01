# Orchestrator in Go


This repo follows the "Orchestrator in Go, From Scratch" (Tim Boring) book.


# Requests to worker api

```bash
curl -v localhost:8901/tasks



uuid=$( uuidgen )
curl -v --request POST \
    -H "Content-Type: application/json" \
    -d "{
        \"id\": \"${uuid}\",
        \"state\": \"running\",
        \"task\": {
            \"state\": \"scheduled\",
            \"id\": \"${uuid}\",
            \"name\": \"test-chapter-5\",
            \"image\": \"strm/helloworld-http\"
        }
    }" \
    localhost:8901/tasks

curl -v --request DELETE \
    localhost:8901/tasks/${uuid}


curl localhost:8901/stats | jq '.'
```


# Todo:
- [x] Chapter 1: Introduction
- [x] Chapter 2: Skeleton Code
- [x] Chapter 3: Task (docker start and stop from code)
- [ ] refactor Docker Result to be Rust results
- Part 2: Worker
- [x] Chapter 4: V0 of Worker
- [x] Chapter 5: Worker API
- [x] Chapter 6: Worker Metrics
- Part 3 Manager:
- [ ] Chapter 7: Manager methods
    - [ ] check invalid transition pending to pending
