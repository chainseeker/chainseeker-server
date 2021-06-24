/**
 * Relays ZeroMQ message from backend as a websocket service.
 */

import * as ZeroMQ from 'zeromq';
import { server as WebSocketServer } from 'websocket';
import * as http from 'http';

export default class WebSocketRelay {
	
	zmq: ZeroMQ.Socket;
	ws: WebSocketServer|null = null;
	
	constructor(zmqPath: string, server: http.Server) {
		// Configure ZeroMQ client.
		this.zmq = ZeroMQ.socket('sub');
		this.zmq.connect(zmqPath);
		this.zmq.subscribe('hashblock');
		this.zmq.subscribe('hashtx');
		this.zmq.on('message', (topic, msg) => {
			//console.log(`[WebSocketRelay.ZMQ] ${topic.toString()} ${msg.toString('hex')}`);
		});
		// Configure WebSocket server.
		this.ws = new WebSocketServer({
			httpServer: server,
		});
		this.ws.on('request', (req) => {
			try {
				const conn = req.accept('chainseeker', req.origin);
				this.zmq.on('message', (topic, msg) => {
					topic = topic.toString();
					msg = msg.toString('hex');
					conn.sendUTF(JSON.stringify([topic, msg]));
				});
			} catch(e) {}
		});
	}
	
}

