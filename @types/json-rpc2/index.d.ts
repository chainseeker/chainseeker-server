
declare module "json-rpc2" {
	
	export class Client {
		
		static $create(port: number, host: string, user: string, password: string): Client;
		call(method: string, params: (string|number|boolean)[], callback: (err: Error, result: any)=>void): void;
		
	}
	
}

