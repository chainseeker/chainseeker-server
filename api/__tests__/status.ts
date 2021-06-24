
import { getCS } from './common.lib';

const cs = getCS();

test('status', async () => {
	const status = await cs.getStatus();
	expect(typeof status.blocks).toBe('number');
});

