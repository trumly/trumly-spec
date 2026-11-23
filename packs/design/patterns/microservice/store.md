---
id: store
type: datastore
parent: service
title: Service data
plane: dsn--design
fields:
  kind: relational
  owner: Team
---
Private to the service: no other service reads or writes it directly. Other services get the data through the API or the events.
