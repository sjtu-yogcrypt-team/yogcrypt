use std::option;
use ::basic::field::*;
use ::basic::cell::yU64x4::*;
use ::basic::cell::yU64x8::*;

const ZERO: yU64x4 =  yU64x4{value:(0,0,0,0)};

pub struct primeField
{
	pub prime: yU64x4,
	pub negPrime: yU64x4,
	pub rho: yU64x4,
	pub rho2: yU64x4,
	pub inv2: yU64x4,
}

macro_rules! OVERFLOWING_ADD
{
	($x:expr, $y:expr, $result:ident, $overflowFlag:ident) => 
	(
		let car = if ($overflowFlag==true) {1} else {0};

		let r1 = u64::overflowing_add($x, $y);
		let r2 = u64::overflowing_add(r1.0, car);
		$result = r2.0;
		$overflowFlag = r1.1|r2.1;
	)
}

impl primeField
{
	pub fn new(prime: yU64x4) -> primeField
	{
		let mut inv2 = prime;

		let mut field = primeField
		{
			prime,
			negPrime: primeField::getNeg(prime),
			rho: yU64x4::new(1,0,0,0),
			rho2: yU64x4::new(1,0,0,0),
			inv2,
		};

		//compute rho
		for i in 0..256
		{
			field.rho.leftShift1();
			if primeField::largerEqualThan(field.rho, prime)
			{
				field.rho = field.rho - prime;
			}
			if(primeField::equalToZero(field.rho)) //overflow when calculate rho
			{
				let mut Rho = !field.prime;
				field.rho = Rho+yU64x4::new(1,0,0,0);
			}
		}

		//compute rho2 (=rho^2 mod p)
		field.rho2 = field.rho;
		for i in 0..256
		{
			let y = field.rho2.value.3 >> 63;
			field.rho2.leftShift1();
			if(y!=0)
			{
				field.rho2 = field.addElement(field.rho2, field.rho);
			}
			if primeField::largerEqualThan(field.rho2, prime)
			{
				field.rho2 = field.rho2 - prime;
			}
		}

		let (inv, b) = field.addElementNoMod(inv2, yU64x4::new(1,0,0,0));
		inv2 = inv;
		inv2.rightShift1();
		field.inv2 = inv2;

		field
	}
	
}

impl theField for primeField
{
	fn getAdditionInverseElement(&self, mut x: yU64x4) -> yU64x4
	{
		self.prime - x
	}
	
	// get the multipilication inverse element of x, or get 0 for 0
	fn getMultiplicationInverseElement(&self, x: yU64x4) -> yU64x4
	{
		// Aborted algorithm: x^-1 = x^(p-2)
/*		let mut T = yU64x4::new(1,0,0,0);
		let mut X = x;
		let mut e = primeField::sub_yU64x4(self.prime,yU64x4::new(2,0,0,0));

		while(!primeField::equalToOne(e))
		{
			if(e.value.0%2==1)
			{
				e.value.0 -= 1;
				T = self.mulElement(T,X);
			}
			else 
			{
				e.rightShift1();
				X = self.mulElement(X,X);
			}
		}
		
		println!("inv1={}",self.mulElement(T,X));*/

		if(primeField::equalToZero(x)) {return yU64x4::new(0,0,0,0);}

		let mut u = x;
		let mut v = self.prime;
		let mut x1 = yU64x4::new(1,0,0,0);
		let mut x2 = yU64x4::new(0,0,0,0);

		while((!primeField::equalToOne(u))&&(!primeField::equalToOne(v)))
		{
			while(u.value.0%2==0)
			{
				u.rightShift1();

				if(x1.value.0%2==0) 
				{
					x1.rightShift1();
				}
				else 
				{
					let (u,overflowFlag) = self.addElementNoMod(x1,self.prime);
					x1 = u;
					x1.rightShift1();
					if(overflowFlag)
					{
						x1.value.3 |= 0x8000000000000000;
					}
				}
			}

			while(v.value.0%2==0)
			{
				v.rightShift1();

				if(x2.value.0%2==0) 
				{
					x2.rightShift1();
				} 
				else 
				{

					let (u,overflowFlag) = self.addElementNoMod(x2,self.prime);
					x2 = u;
					x2.rightShift1();
					if(overflowFlag)
					{
						x2.value.3 |= 0x8000000000000000;
					}
				}
			}

			if(primeField::largerEqualThan(u,v))
			{
				u = self.subElement(u,v);
				x1 = self.subElement(x1,x2);
			}
			else 
			{
				v = self.subElement(v,u);
				x2 = self.subElement(x2,x1);
			}
		}

		if(primeField::equalToOne(u))
		{
			while(primeField::largerEqualThan(x1,self.prime))
			{
				x1 = x1 - self.prime;
			}
			x1
		}
		else
		{
			while(primeField::largerEqualThan(x2, self.prime))
			{
				x2 = x2 - self.prime;
			}
			x2
		}
	}

	fn addElement(&self, x: yU64x4, y: yU64x4) -> yU64x4
	{
		let res0: u64;
		let res1: u64;
		let res2: u64;
		let res3: u64;
		let mut overflowFlag = false;

		OVERFLOWING_ADD!(x.value.0, y.value.0, res0, overflowFlag);
		OVERFLOWING_ADD!(x.value.1, y.value.1, res1, overflowFlag);
		OVERFLOWING_ADD!(x.value.2, y.value.2, res2, overflowFlag);
		OVERFLOWING_ADD!(x.value.3, y.value.3, res3, overflowFlag);
		

		let mut m = yU64x4
		{
			value: (res0, res1, res2, res3),
		};

		if overflowFlag==true  //overflow
		{
			m = self.addElement(self.rho, m);
		} 

		if primeField::largerEqualThan(m,self.prime)
		{ m - self.prime }
		else 
		{ m }
	}

	fn subElement(&self, x: yU64x4, y: yU64x4) -> yU64x4
	{
		self.addElement(x, self.getAdditionInverseElement(y))
	}

	fn mulElement(&self, x: yU64x4, y: yU64x4) -> yU64x4
	{
		let x_bar = self.montMul(x, self.rho2);
		let y_bar = self.montMul(y, self.rho2);
		let t_bar = self.montMul(x_bar, y_bar);
		self.montRed(t_bar)
	}

	fn divElement(&self, x: yU64x4, y: yU64x4) -> yU64x4
	{	
		let q = self.getMultiplicationInverseElement(y);
		self.mulElement(x, q)
	}

/*	fn sqrtElement(&self, x: yU64x4) -> yU64x4
	{
		
	}*/
}

// Mong
impl primeField
{
	pub fn transformToElement(&self, mut x: yU64x4) -> yU64x4
	{
		while(primeField::largerEqualThan(x,self.prime))
		{
			x = x - self.prime;
		}

		x
	}

	// get x * y * 2^(-256) mod p
	pub fn montMul(&self, x: yU64x4, y:yU64x4) -> yU64x4
	{
		let mut z = yU64x4::new(0, 0, 0, 0);


		for i in 0..256
		{
			z = if(y.get(i)==1) 
			{
				self.addElement(z,x)
			} 
			else 
			{
				z
			} ;

			if(z.value.0%2==1) 
			{
				let (u,overflowFlag) = self.addElementNoMod(z, self.prime);
				z = u;
				z.rightShift1();
				if(overflowFlag)
				{
					z.value.3 |= 0x8000000000000000;
				}
			}
			else 
			{
				z.rightShift1();
			}

			
		};

		if(primeField::largerEqualThan(z, self.prime)) {z - self.prime} else {z}
	}

	// get t * 2^(-256) mod p
	pub fn montRed(&self, mut t: yU64x4) -> yU64x4
	{
		for i in 0..256
		{
			if(t.value.0%2==1) 
			{
				let (u,overflowFlag) = self.addElementNoMod(t, self.prime);
				t = u;
				t.rightShift1();
				if(overflowFlag)
				{
					t.value.3 |= 0x8000000000000000;
				}
			}	
			else
			{
				t.rightShift1();
			}
		}

		if(primeField::largerEqualThan(t, self.prime)) {self.subElement(t, self.prime)} else {t}
	}
}

// Arithmetic
impl primeField
{
	fn getNeg(mut x: yU64x4) -> yU64x4
	{

		if x.value.0!=0
		{
			x.value.0 = u64::wrapping_neg(x.value.0);
			x.value.1 = !x.value.1;
			x.value.2 = !x.value.2;
			x.value.3 = !x.value.3;
		}
		else if x.value.1!=0
		{
			x.value.1 = u64::wrapping_neg(x.value.1);
			x.value.2 = !x.value.2;
			x.value.3 = !x.value.3;
		}
		else if x.value.2!=0
		{
			x.value.2 = u64::wrapping_neg(x.value.2);
			x.value.3 = !x.value.3;
		}
		else if x.value.3!=0
		{
			x.value.3 = u64::wrapping_neg(x.value.3);
		}

		x
	}
}

// Order
impl primeField
{
	pub fn largerEqualThan(x: yU64x4, y: yU64x4) -> bool
	{
		if(x.value.3>y.value.3) {return true;};
		if(x.value.3<y.value.3) {return false;};
		if(x.value.2>y.value.2) {return true;};
		if(x.value.2<y.value.2) {return false;};
		if(x.value.1>y.value.1) {return true;};
		if(x.value.1<y.value.1) {return false;};
		if(x.value.0>=y.value.0) {return true;};
		return false;
	}

	pub fn equalTo(x: yU64x4, y: yU64x4) -> bool
	{
		x.value.0==y.value.0 && x.value.1==y.value.1 && x.value.2==y.value.2 && x.value.3==y.value.3
	}

	pub fn equalToZero(x: yU64x4) -> bool
	{
		x.value.0==0 && x.value.1==0 && x.value.2==0 && x.value.3==0
	}

	pub fn equalToOne(x: yU64x4) -> bool
	{
		x.value.0==1 && x.value.1==0 && x.value.2==0 && x.value.3==0
	}
}

impl primeField
{
	pub fn addElementNoMod(&self, x: yU64x4, y: yU64x4) -> (yU64x4,bool)
	{
		let res0: u64;
		let res1: u64;
		let res2: u64;
		let res3: u64;
		let mut overflowFlag = false;

		OVERFLOWING_ADD!(x.value.0, y.value.0, res0, overflowFlag);
		OVERFLOWING_ADD!(x.value.1, y.value.1, res1, overflowFlag);
		OVERFLOWING_ADD!(x.value.2, y.value.2, res2, overflowFlag);
		OVERFLOWING_ADD!(x.value.3, y.value.3, res3, overflowFlag);
		
		let mut m = yU64x4
		{
			value: (res0, res1, res2, res3),
		};

		(m,overflowFlag)
	}

	pub fn div2(&self, mut x: yU64x4) -> yU64x4
	{
		if(x.value.0%2==1)
		{
			x = self.addElement(x, self.prime);
		}
		x.rightShift1();

		x
	}
}