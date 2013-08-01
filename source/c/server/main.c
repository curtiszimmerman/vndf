#include <assert.h>
#include <limits.h>
#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include <netdb.h>
#include <sys/epoll.h>
#include <unistd.h>

#include <common/idmap.h>
#include <common/rbuf.h>
#include <common/stack.h>
#include "clients.h"
#include "events.h"
#include "net.h"


#define PI 3.141592653589793238462


void logOutput(char *s);

void onConnect(int clientFD, clientMap *clientMap);
void onDisconnect(size_t clientId, clientMap *clientMap, events *events);
void onUpdate(clientMap *clientMap, events *events, double dTimeInS);
int sendUpdate(int clientFD, size_t id, double xPos, double yPos);
int sendRemove(int clientFD, size_t id);


int main(int argc, char const *argv[])
{
	logOutput("Core Service started.\n");

	srand((unsigned int)time(NULL));

	net net = net_init("34481");

	events events;
	rbuf_init(events, 16);

	clientMap clientMap;
	clients_initClientMap(&clientMap, 4);

	while (true)
	{
		const int frameTimeInMs = 50;

		const int maxEvents = 1024;
		struct epoll_event pollEvents[maxEvents];

		int numberOfEvents = epoll_wait(
			net.pollerFD,
			pollEvents,
			maxEvents,
			frameTimeInMs);
		assert(numberOfEvents != -1);

		for (int i = 0; i < numberOfEvents; i += 1)
		{
			int clientFD = net_acceptClient(net.serverFD);

			event connectEvent;
			connectEvent.type = ON_CONNECT;
			connectEvent.ev.onConnect.clientFD = clientFD;

			rbuf_put(events, connectEvent);
		}

		event updateEvent;
		updateEvent.type = ON_UPDATE;

		rbuf_put(events, updateEvent);

		while (rbuf_size(events) > 0)
		{
			event event;
			rbuf_get(events, &event);

			switch (event.type)
			{
				case ON_CONNECT:
					onConnect(event.ev.onConnect.clientFD, &clientMap);
					break;

				case ON_DISCONNECT:
					onDisconnect(
						event.ev.onDisconnect.clientId,
						&clientMap,
						&events);
					break;

				case ON_UPDATE:
					onUpdate(
						&clientMap,
						&events,
						(double)frameTimeInMs / 1000.0);
					break;

				default:
					assert(false);
			}
		}
	}
}

void logOutput(char *s)
{
	time_t t = time(NULL);
	char *ts = ctime(&t);
	ts[strlen(ts) - 1] = '\0';

	printf("%s  %s", ts, s);
}

void onConnect(int clientFD, clientMap *clientMap)
{
	if (clients_canAdd(clientMap))
	{
		double distance = 100.0;

		double alpha = 90.0 / 180.0 * PI;

		double xPos = distance * cos(alpha);
		double yPos = distance * sin(alpha);

		double speed = 30;

		clients_add(clientMap, clientFD, (vec2){xPos, yPos}, (vec2){speed, 0});
	}
	else
	{
		int status = close(clientFD);
		assert(status == 0);
	}
}

void onDisconnect(size_t clientId, clientMap *clientMap, events *events)
{
	clients_remove(clientMap, clientId);

	idmap_each(clientMap->clients, i,
		int status = sendRemove(
			idmap_get(clientMap->clients, i).socketFD,
			clientId);

		if (status < 0)
		{
			event disconnectEvent;
			disconnectEvent.type = ON_DISCONNECT;
			disconnectEvent.ev.onDisconnect.clientId = i;

			rbuf_put((*events), disconnectEvent);
		}
	)
}

void onUpdate(clientMap *clientMap, events *events, double dTimeInS)
{
	idmap_each(clientMap->clients, i,
		client *client = &idmap_get(clientMap->clients, i);
		body *ship = &client->ship;

		double gMag = 3000 / vec_magnitude(ship->pos);
		vec2 g = vec_scale(vec_normalize(ship->pos), -gMag);

		ship->pos = vec_add(ship->pos, vec_scale(ship->vel, dTimeInS));
		ship->vel = vec_add(ship->vel, vec_scale(g, dTimeInS));
	)

	idmap_each(clientMap->clients, i,
		idmap_each(clientMap->clients, j,
			int status = sendUpdate(
				idmap_get(clientMap->clients, i).socketFD,
				idmap_get(clientMap->clients, j).id,
				idmap_get(clientMap->clients, j).ship.pos.x,
				idmap_get(clientMap->clients, j).ship.pos.y);

			if (status < 0)
			{
				event disconnectEvent;
				disconnectEvent.type = ON_DISCONNECT;
				disconnectEvent.ev.onDisconnect.clientId = i;

				rbuf_put((*events), disconnectEvent);
			}
		)
	)
}

int sendUpdate(int clientFD, size_t id, double xPos, double yPos)
{
	char message[256];
	int status = snprintf(
		message + 1, sizeof message - 1,
		"UPDATE id: %lu, pos: (%f, %f)",
		id, xPos, yPos);
	assert(status >= 0);
	assert((size_t)status <= sizeof message);

	size_t messageLength = strlen(message + 1) + 1;
	assert(messageLength <= CHAR_MAX);
	message[0] = (char)messageLength;

	return net_send(clientFD, message, messageLength);
}

int sendRemove(int clientFD, size_t id)
{
	char message[256];
	int status = snprintf(
		message + 1, sizeof message - 1,
		"REMOVE id: %lu",
		id);
	assert(status >= 0);
	assert((size_t)status <= sizeof message);

	size_t messageLength = strlen(message + 1) + 1;
	assert(messageLength <= CHAR_MAX);
	message[0] = (char)messageLength;

	return net_send(clientFD, message, messageLength);
}
