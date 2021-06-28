
import { Chainseeker } from 'chainseeker';

export const getCS = () => {
	return new Chainseeker('http://localhost:7001/api');
}

