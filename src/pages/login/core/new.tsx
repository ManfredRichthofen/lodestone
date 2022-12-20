import axios, { AxiosError } from 'axios';
import Button from 'components/Atoms/Button';
import { useRouter } from 'next/router';
import { useContext, useEffect, useState } from 'react';
import { errorToString, LODESTONE_PORT, pushKeepQuery } from 'utils/util';
import { NextPageWithLayout } from '../../_app';
import Link from 'next/link';
import { CoreConnectionInfo, LodestoneContext } from 'data/LodestoneContext';
import { ClientError } from 'bindings/ClientError';
import { LoginReply } from 'bindings/LoginReply';
import InputField from 'components/Atoms/Form/InputField';
import { Form, Formik, FormikHelpers } from 'formik';
import * as yup from 'yup';
import ComboField from 'components/Atoms/Form/ComboField';
import SelectField from 'components/Atoms/Form/SelectField';
import { faArrowLeft, faArrowRight } from '@fortawesome/free-solid-svg-icons';

const validationSchema = yup.object({
  address: yup.string().required('Required'),
  port: yup.number().required('Required'),
  apiVersion: yup.string().required('Required'),
  protocol: yup.string().required('Required'),
});

const Auth: NextPageWithLayout = () => {
  const router = useRouter();
  const { token, setToken, tokens, isReady, core, setCore } =
    useContext(LodestoneContext);
  const { address, port, apiVersion, protocol } = core;
  const coreList = tokens ? Object.keys(tokens) : [];

  const initialValues: CoreConnectionInfo = {
    address: '',
    port: LODESTONE_PORT.toString(),
    apiVersion: 'v1',
    protocol: 'http', //change to https when supported
  };

  const onSubmit = (
    values: CoreConnectionInfo,
    actions: FormikHelpers<CoreConnectionInfo>
  ) => {
    // check if core can be reached
    axios
      .get(`/info`, {
        baseURL: `${values.protocol}://${values.address}:${values.port}/api/${values.apiVersion}`,
      })
      .then((res) => {
        if (res.status !== 200)
          throw new Error('Invalid response, setup may be invalid');
      })
      .then(() => {
        setCore(values, '/login/user/select');
        actions.setSubmitting(false);
      })
      .catch((err) => {
        const errorMessages = errorToString(err);
        actions.setErrors({ address: errorMessages }); //TODO: put the error in a better place, it's not just an address problem
        actions.setSubmitting(false);
        return;
      });
  };

  return (
    <div
      className="flex h-screen flex-col items-center justify-center p-8"
      style={{
        background: "url('/login_background.svg')",
        backgroundSize: 'cover',
      }}
    >
      <div className="flex w-[768px] max-w-full flex-col items-stretch justify-center gap-12 rounded-3xl bg-gray-850 px-14 py-20 @container">
        <div className="text flex flex-col items-start">
          <img src="/logo.svg" alt="logo" className="h-9 w-40" />
          <h1 className=" font-title text-2xlarge font-medium tracking-medium text-gray-300">
            Add a new core
          </h1>
        </div>
        {isReady ? (
          <Formik
            initialValues={initialValues}
            validationSchema={validationSchema}
            onSubmit={onSubmit}
            validateOnBlur={false}
            validateOnChange={false}
          >
            {({ isSubmitting }) => (
              <Form
                id="addCoreForm"
                className="flex flex-col gap-12"
                autoComplete="nope"
              >
                <div className="grid h-32 grid-cols-1 gap-y-14 gap-x-8 @lg:grid-cols-2">
                  <SelectField
                    name="apiVersion"
                    className="grow"
                    options={['v1']}
                    label="API Version"
                  />
                  <SelectField
                    name="protocol"
                    className="grow"
                    options={['http', 'https']}
                    label="Protocol"
                  />
                  <InputField type="number" name="port" label="Port" />
                  <InputField
                    type="text"
                    name="address"
                    label="IP Address/Domain"
                    placeholder='e.g. 123.123.123.123 or "myserver.com"'
                  />
                </div>
                <div className="flex w-full flex-row justify-end gap-4">
                  <Button
                    icon={faArrowLeft}
                    label="Back"
                    onClick={() => router.back()}
                  />
                  <Button
                    type="submit"
                    color="primary"
                    label="Add and Continue"
                    icon={faArrowRight}
                    loading={isSubmitting} //TODO: fix button size changing when loading
                  />
                </div>
              </Form>
            )}
          </Formik>
        ) : (
          <div className="h-32">
            <p>Loading...</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default Auth;
