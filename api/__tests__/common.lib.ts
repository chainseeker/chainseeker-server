
import * as cs from 'chainseeker-lib';
cs.setApiEndPoint('http://localhost:7001/api');

export const getCS = () => {
	return cs;
}

