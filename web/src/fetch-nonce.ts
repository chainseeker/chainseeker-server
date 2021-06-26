
import { Chainseeker } from 'chainseeker';

const main = async () => {
	const COIN_NAME = process.argv[2];
	const configs = (await import(`${process.env['HOME']}/.chainseeker/config.ts`)).configs;
	const config = configs[COIN_NAME];
	const cs = new Chainseeker(config.endpoint.apiLocal);
	cs.getBlockSummary(0, 0x7fffffff, ['nonce']).then((data) => {
		console.log(data.map((d, i) => `${i} ${d.nonce}`).join('\n'));
	});
};

main();

